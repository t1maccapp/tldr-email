use tokio::sync::RwLock;

#[derive(Debug, Clone)]
pub struct Account {
    pub login: String,
    pub password: String,
}

#[derive(Debug, Default)]
pub struct TerminalState {
    pub accounts: Vec<Account>,
}

#[derive(Debug, Default)]
pub struct State {
    pub accounts: RwLock<Vec<Account>>,
}

impl State {
    pub async fn add_account(&self, login: String, password: String) {
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
