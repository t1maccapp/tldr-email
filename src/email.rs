use email::{
    account::config::passwd::PasswordConfig,
    autoconfig::{
        config::{AutoConfig, SecurityType, ServerType},
        from_addr,
    },
    backend::Backend,
    envelope::{
        list::{ListEnvelopes, ListEnvelopesOptions},
        Id,
    },
    folder::list::ListFolders,
    imap::ImapContext,
    message::{get::GetMessages, send::SendMessageThenSaveCopy, Message},
    smtp::{
        config::{SmtpAuthConfig, SmtpConfig},
        SmtpContextBuilder, SmtpContextSync,
    },
};
use email_address::EmailAddress;
use std::{
    collections::{HashMap, HashSet},
    sync::Arc,
    time::Duration,
};
use tokio_stream::wrappers::UnboundedReceiverStream;

use anyhow::{bail, Result};

use crate::{
    actions::Actions,
    state::{Account, State},
};
use email::{
    account::config::AccountConfig,
    backend::BackendBuilder,
    imap::{
        config::{ImapAuthConfig, ImapConfig},
        ImapContextBuilder,
    },
    tls::Encryption,
};
use futures::StreamExt;
use tokio::{
    sync::{
        mpsc::{self, UnboundedReceiver, UnboundedSender},
        RwLock,
    },
    time::sleep,
};

pub struct EmailBackend {
    account_to_backends_map: HashMap<String, (Backend<ImapContext>, Backend<SmtpContextSync>)>,
    tx: UnboundedSender<Actions>,
    rx: UnboundedReceiver<Actions>,
}

impl EmailBackend {
    pub async fn new(state: Arc<State>) -> Result<Self> {
        let mut account_to_backends_map = HashMap::new();

        for account in state.accounts.read().await.iter() {
            println!("Loading autoconfig for {}", account.login);

            let autoconfig = from_addr(account.login.clone()).await?;

            let imap_backend = Self::build_imap(account, &autoconfig).await;
            let smtp_backend = Self::build_smtp(account, &autoconfig).await;
            account_to_backends_map.insert(account.login.clone(), (imap_backend?, smtp_backend?));

            println!("{} – done", account.login);
        }

        let (tx, rx) = mpsc::unbounded_channel();

        sleep(Duration::from_millis(400)).await;

        Ok(Self {
            account_to_backends_map,
            tx,
            rx,
        })
    }

    // Will only be ran once due to consuming itself
    // TODO: maybe add an Once guard
    pub async fn spawn(self, state: Arc<State>) -> Result<UnboundedSender<Actions>> {
        let mut rx = UnboundedReceiverStream::new(self.rx);
        let account_to_backends_map = self.account_to_backends_map;
        let tx = self.tx;

        let debouncer = Arc::new(RwLock::new(HashSet::<Actions>::new()));
        let debouncer_cloned = debouncer.clone();

        tokio::task::spawn(async move {
            while let Some(action) = rx.next().await {
                debouncer.write().await.replace(action);
                eprintln!("debouncer len = {}", debouncer.read().await.len());
            }
        });

        tokio::task::spawn(async move {
            loop {
                for action in debouncer_cloned.write().await.drain() {
                    Self::execute_action(&account_to_backends_map, action, state.clone()).await;
                }

                // throttle actions
                sleep(Duration::from_millis(500)).await;

                if debouncer_cloned.write().await.len() == 0 {
                    *state.is_updating.write().await = false;
                }
            }
        });

        Ok(tx)
    }

    async fn build_imap(
        account: &Account,
        autoconfig: &AutoConfig,
    ) -> Result<Backend<ImapContext>> {
        let account_config = Arc::new(AccountConfig::default());

        let imap_server = autoconfig
            .email_provider()
            .incoming_servers()
            .iter()
            .find(|s| matches!(s.server_type(), ServerType::Imap))
            .copied();

        let Some(imap_server) = imap_server else {
            bail!(
                "No IMAP server auto configuration found for {}",
                account.login
            );
        };

        let Some(imap_host) = imap_server.hostname() else {
            bail!(
                "No IMAP server hostname in autoconfig for {}",
                account.login
            )
        };

        let Some(imap_port) = imap_server.port() else {
            bail!("No IMAP server port in autoconfig for {}", account.login)
        };

        let Some(imap_encryption) = imap_server.security_type() else {
            bail!("No IMAP security type in autoconfig for {}", account.login)
        };

        if !matches!(imap_encryption, SecurityType::Tls) {
            bail!(
                "IMAP server does not support Tls security in autoconfig for {}",
                account.login
            )
        };

        let imap_config = Arc::new(ImapConfig {
            host: imap_host.to_string(),
            port: *imap_port,
            encryption: Some(Encryption::default()),
            login: account.login.to_string(),
            auth: ImapAuthConfig::Password(PasswordConfig(account.password.clone())),
            ..Default::default()
        });

        let imap_ctx = ImapContextBuilder::new(account_config.clone(), imap_config.clone());

        let imap = BackendBuilder::new(account_config, imap_ctx).build().await;

        let Ok(imap) = imap else {
            bail!("IMAP backend cannot be created for {}", account.login)
        };

        Ok(imap)
    }

    async fn build_smtp(
        account: &Account,
        autoconfig: &AutoConfig,
    ) -> Result<Backend<SmtpContextSync>> {
        let account_config = Arc::new(AccountConfig::default());

        let smtp_server = autoconfig
            .email_provider()
            .outgoing_servers()
            .iter()
            .find(|s| matches!(s.server_type(), ServerType::Smtp))
            .copied();

        let Some(smtp_server) = smtp_server else {
            bail!(
                "No SMTP server auto configuration found for {}",
                account.login
            );
        };

        let Some(smtp_host) = smtp_server.hostname() else {
            bail!(
                "No SMTP server hostname in autoconfig for {}",
                account.login
            )
        };

        let Some(smtp_port) = smtp_server.port() else {
            bail!("No SMTP server port in autoconfig for {}", account.login)
        };

        let Some(smtp_encryption) = smtp_server.security_type() else {
            bail!("No SMTP security type in autoconfig for {}", account.login)
        };

        if !matches!(smtp_encryption, SecurityType::Tls) {
            bail!(
                "SMTP server does not support Tls security in autoconfig for {}",
                account.login
            )
        };

        let smtp_config = Arc::new(SmtpConfig {
            host: smtp_host.to_string(),
            port: *smtp_port,
            encryption: Some(Encryption::default()),
            login: account.login.to_string(),
            auth: SmtpAuthConfig::Password(PasswordConfig(account.password.clone())),
            ..Default::default()
        });

        let smtp_ctx = SmtpContextBuilder::new(account_config.clone(), smtp_config.clone());

        let smtp = BackendBuilder::new(account_config, smtp_ctx).build().await;

        let Ok(smtp) = smtp else {
            bail!("SMTP backend cannot be created for {}", account.login)
        };

        Ok(smtp)
    }

    async fn execute_action(
        account_map: &HashMap<String, (Backend<ImapContext>, Backend<SmtpContextSync>)>,
        action: Actions,
        state: Arc<State>,
    ) {
        match action {
            Actions::ListFolders { login } => {
                state
                    .account_folders
                    .write()
                    .await
                    .insert(login.clone(), None);

                let Some(backends) = account_map.get(&login) else {
                    return;
                };

                let Ok(folders) = backends.0.list_folders().await else {
                    return;
                };

                state.account_folders.write().await.insert(
                    login,
                    Some(folders.iter().map(|f| f.name.clone()).collect()),
                );
            }

            Actions::ListEnvelopes {
                login,
                folder,
                page,
            } => {
                state
                    .account_envelopes
                    .write()
                    .await
                    .insert(login.clone(), None);

                let Some(backends) = account_map.get(&login) else {
                    return;
                };

                let Ok(envelopes) = backends
                    .0
                    .list_envelopes(
                        &folder,
                        ListEnvelopesOptions {
                            page_size: 10,
                            page,
                            query: None,
                        },
                    )
                    .await
                else {
                    return;
                };

                state
                    .account_envelopes
                    .write()
                    .await
                    .insert(login, Some(envelopes.to_vec()));
            }

            Actions::GetMessage { login, folder, id } => {
                *state.message.write().await = None;

                let Some(backends) = account_map.get(&login) else {
                    return;
                };

                let messages = backends
                    .0
                    .get_messages(&folder, &Id::Single(id.into()))
                    .await;

                let Ok(messages) = messages else {
                    return;
                };

                let Some(message) = messages.first() else {
                    return;
                };

                let Ok(parsed_message) = message.parsed() else {
                    return;
                };

                let Some(body_text) = parsed_message.body_text(0) else {
                    return;
                };

                *state.message.write().await = Some(body_text.to_string());
            }

            Actions::SendMessage {
                login,
                to,
                subject,
                text,
            } => {
                let Some(backends) = account_map.get(&login) else {
                    return;
                };

                if !EmailAddress::is_valid(&to) {
                    return;
                }

                let msg = [
                    "Content-Type: text/plain",
                    &format!("From: {}", login).to_string(),
                    &format!("To: {}", to).to_string(),
                    &format!("Subject: {}", subject).to_string(),
                    "",
                    &text,
                ]
                .join("\n");

                let msg = Message::from(msg.as_str());

                let Ok(raw) = msg.raw() else {
                    return;
                };

                let _ = backends.1.send_message_then_save_copy(raw).await;
            }
        }
    }
}
