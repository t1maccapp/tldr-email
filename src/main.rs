mod actions;
mod app;
mod args;
mod email;
mod state;
mod ui;

use std::sync::Arc;

use anyhow::Result;
use app::App;
use args::get_initial_state_from_args;
use email::EmailBackend;
use state::State;

#[tokio::main]
async fn main() -> Result<()> {
    let mut terminal = ratatui::init();

    let state = get_initial_state_from_args().await?;

    spawn_email_backend_task(state.clone()).await?;

    let app_result = App::default().run(&mut terminal, state.clone()).await;

    ratatui::restore();

    app_result
}

async fn spawn_email_backend_task(state: Arc<State>) -> Result<()> {
    let email_backend = EmailBackend::new(state).await?;

    Ok(())
}
