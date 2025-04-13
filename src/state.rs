use secret::Secret;
use tokio::sync::{mpsc::UnboundedSender, RwLock};

use crate::actions::Actions;

#[derive(Debug, Clone)]
pub struct Account {
    pub login: String,
    pub password: Secret,
}

#[derive(Debug, Default)]
pub struct TerminalState {
    pub accounts: Vec<Account>,
}

#[derive(Debug, Default)]
pub struct State {
    pub accounts: RwLock<Vec<Account>>,
    email_backend_tx: Option<RwLock<UnboundedSender<Actions>>>,
}

impl State {
    pub async fn add_account(&self, login: String, password: Secret) {
        self.accounts
            .write()
            .await
            .push(Account { login, password });
    }

    pub async fn as_terminal_state(&self) -> TerminalState {
        TerminalState {
            accounts: self.accounts.read().await.clone(),
        }
    }
}
