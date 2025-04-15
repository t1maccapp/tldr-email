mod actions;
mod app;
mod args;
mod email;
mod state;
mod ui;

use std::sync::Arc;

use actions::Actions;
use anyhow::Result;
use app::App;
use args::get_initial_state_from_args;
use email::EmailBackend;
use state::State;
use tokio::sync::mpsc::UnboundedSender;

#[tokio::main]
async fn main() -> Result<()> {
    let state = get_initial_state_from_args().await?;
    let actions_tx = spawn_email_backend_task(state.clone()).await?;

    let mut terminal = ratatui::init();
    let app_result = App::default()
        .run(&mut terminal, state.clone(), actions_tx)
        .await;
    ratatui::restore();
    app_result
}

async fn spawn_email_backend_task(state: Arc<State>) -> Result<UnboundedSender<Actions>> {
    let email_backend = EmailBackend::new(state.clone()).await?;

    let email_backend_tx = email_backend.spawn(state.clone()).await?;

    Ok(state.spawn_email_action_forwarder(email_backend_tx).await)
}
