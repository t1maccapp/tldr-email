use email::{
    account::config::passwd::PasswordConfig,
    autoconfig::{
        config::{AutoConfig, SecurityType, ServerType},
        from_addr,
    },
    backend::Backend,
    imap::ImapContext,
    smtp::{
        config::{SmtpAuthConfig, SmtpConfig},
        SmtpContextBuilder, SmtpContextSync,
    },
};
use futures::StreamExt;
use std::{collections::HashMap, sync::Arc, time::Duration};
use tokio_stream::wrappers::UnboundedReceiverStream;

use anyhow::{bail, Result};

use email::{
    account::config::AccountConfig,
    backend::BackendBuilder,
    imap::{
        config::{ImapAuthConfig, ImapConfig},
        ImapContextBuilder,
    },
    tls::Encryption,
};
use tokio::{
    sync::mpsc::{self, UnboundedReceiver, UnboundedSender},
    time::sleep,
};

use crate::{
    actions::Actions,
    state::{Account, State},
};

pub struct EmailBackend {
    account_to_backends_map: HashMap<String, (Backend<ImapContext>, Backend<SmtpContextSync>)>,
    tx: Arc<UnboundedSender<Actions>>,
    rx: UnboundedReceiver<Actions>,
}

impl EmailBackend {
    pub async fn new(state: Arc<State>) -> Result<Self> {
        let mut account_to_backends_map = HashMap::new();

        for account in state.accounts.read().await.iter() {
            let autoconfig = from_addr(account.login.clone()).await?;

            let imap_backend = Self::build_imap(account, &autoconfig).await;

            let smtp_backend = Self::build_smtp(account, &autoconfig).await;

            account_to_backends_map.insert(account.login.clone(), (imap_backend?, smtp_backend?));
        }

        sleep(Duration::from_secs(30000)).await;

        let (tx, rx) = mpsc::unbounded_channel();

        Ok(Self {
            account_to_backends_map,
            tx: Arc::new(tx),
            rx,
        })
    }

    // Will only be ran once due to consuming itself
    // TODO: maybe add a once guard
    pub async fn spawn(self) -> Result<Arc<UnboundedSender<Actions>>> {
        let mut rx = UnboundedReceiverStream::new(self.rx);

        tokio::task::spawn(async move {
            while let Some(action) = rx.next().await {
                println!("{:?}", action);
            }
        });

        Ok(self.tx.clone())
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
            .incoming_servers()
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
}
