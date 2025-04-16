use std::sync::Arc;

use crate::state::State;

use clap::Parser;

use anyhow::Result;
use secret::Secret;

#[derive(Parser)]
pub struct Args {
    #[clap(alias = "account", short, long, value_parser, num_args = 1..)]
    pub accounts: Vec<String>,
}

pub async fn get_initial_state_from_args() -> Result<Arc<State>> {
    let state = Arc::new(State::default());

    let args = Args::parse();

    if args.accounts.is_empty() {
        anyhow::bail!("provide at least one --account login:password");
    }

    for account_str in args.accounts {
        let parts: Vec<&str> = account_str.split(':').collect();

        if parts.len() != 2 {
            anyhow::bail!("invalid account format. Expected 'login:password'");
        }

        state
            .add_account(parts[0].to_string(), Secret::new_raw(parts[1].to_string()))
            .await;
    }

    Ok(state)
}
