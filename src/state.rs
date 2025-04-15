use std::{collections::HashMap, sync::Arc};

use email::envelope::Envelope;
use secret::Secret;
use tokio::sync::{
    mpsc::{self, UnboundedSender},
    RwLock,
};
use tokio_stream::{wrappers::UnboundedReceiverStream, StreamExt};

use crate::actions::Actions;

#[derive(Debug, Clone)]
pub struct Account {
    pub login: String,
    pub password: Secret,
}

#[derive(Debug, Default)]
pub struct ViewState {
    pub accounts: Vec<String>,
    pub folders: Option<Vec<String>>,
    pub messages: Option<Vec<Envelope>>,
}

#[derive(Debug, Default)]
pub struct State {
    pub accounts: RwLock<Vec<Account>>,
    pub account_folders: RwLock<HashMap<String, Option<Vec<String>>>>,
    pub account_envelopes: RwLock<HashMap<String, Option<Vec<Envelope>>>>,
    pub is_updating: Arc<RwLock<bool>>,
    email_backend_tx: Arc<RwLock<Option<UnboundedSender<Actions>>>>,
}

impl State {
    pub async fn add_account(&self, login: String, password: Secret) {
        self.accounts
            .write()
            .await
            .push(Account { login, password });
    }

    pub async fn spawn_email_action_forwarder(
        &self,
        tx: UnboundedSender<Actions>,
    ) -> UnboundedSender<Actions> {
        *self.email_backend_tx.write().await = Some(tx);

        let email_backend_tx = self.email_backend_tx.clone();

        let (sync_tx, rx) = mpsc::unbounded_channel();
        let mut rx = UnboundedReceiverStream::new(rx);

        tokio::task::spawn(async move {
            while let Some(action) = rx.next().await {
                eprintln!("{:?}", action);
                let _ = email_backend_tx.read().await.as_ref().unwrap().send(action);
            }
        });

        sync_tx
    }

    pub async fn is_updating(&self) -> bool {
        *self.is_updating.read().await
    }

    pub async fn as_view_state(&self, login: Option<String>) -> ViewState {
        if let Some(login) = login {
            ViewState {
                accounts: self
                    .accounts
                    .read()
                    .await
                    .iter()
                    .map(|a| a.login.clone())
                    .collect(),
                folders: self
                    .account_folders
                    .read()
                    .await
                    .get(&login)
                    .unwrap_or(&None)
                    .clone(),
                messages: self
                    .account_envelopes
                    .read()
                    .await
                    .get(&login)
                    .unwrap_or(&None)
                    .clone(),
            }
        } else {
            ViewState {
                accounts: self
                    .accounts
                    .read()
                    .await
                    .iter()
                    .map(|a| a.login.clone())
                    .collect(),
                folders: None,
                messages: None,
            }
        }
    }
}
