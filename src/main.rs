mod app;
mod args;
mod email;
mod state;
mod ui;

use anyhow::Result;
use app::App;
use args::get_initial_state_from_args;

#[tokio::main]
async fn main() -> Result<()> {
    let mut terminal = ratatui::init();

    let state = get_initial_state_from_args().await?;

    let app_result = App::default().run(&mut terminal, state.clone()).await;

    ratatui::restore();

    app_result
}
