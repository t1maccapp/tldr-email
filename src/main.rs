mod app;
mod state;
mod ui;

use std::sync::Arc;

use anyhow::Result;
use app::App;
use clap::Parser;
use state::State;

#[derive(Parser)]
pub struct Args {
    #[clap(alias = "account", short, long, value_parser, num_args = 1..)]
    pub accounts: Vec<String>,
}

#[tokio::main]
async fn main() -> Result<()> {
    let state = Arc::new(State::default());

    let args = Args::parse();

    if args.accounts.len() < 1 {
        anyhow::bail!("provide at least one --account login:password");
    }

    for account_str in args.accounts {
        let parts: Vec<&str> = account_str.split(':').collect();

        if parts.len() != 2 {
            anyhow::bail!("invalid account format. Expected 'login:password'");
        }

        state
            .add_account(parts[0].to_string(), parts[1].to_string())
            .await;
    }

    let mut terminal = ratatui::init();

    let app_result = App::default().run(&mut terminal, state.clone()).await;

    ratatui::restore();

    app_result
}
