use std::sync::Arc;

use anyhow::Result;
use clap::Parser;
use crossterm::event::{self, Event, KeyCode, KeyEvent, KeyEventKind};
use ratatui::{
    layout::{Constraint, Direction, Layout},
    style::{Style, Stylize},
    widgets::{Block, List},
    DefaultTerminal, Frame,
};
use tokio::sync::RwLock;

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

#[derive(Debug, Clone)]
pub struct Account {
    login: String,
    password: String,
}

#[derive(Debug, Default)]
pub struct State {
    pub accounts: RwLock<Vec<Account>>,
}

#[derive(Debug, Default)]
pub struct TerminalState {
    pub accounts: Vec<Account>,
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

#[derive(Debug, Default)]
pub enum SelectedWidget {
    Accounts,
    Folders,
    #[default]
    Messages,
}

#[derive(Debug, Default)]
pub struct App {
    selected_widget: SelectedWidget,
    state: TerminalState,
    exit: bool,
}

impl App {
    pub async fn run(&mut self, terminal: &mut DefaultTerminal, state: Arc<State>) -> Result<()> {
        while !self.exit {
            let terminal_state = state.as_terminal_state().await;

            terminal.draw(|frame| self.draw(frame, terminal_state))?;
            self.handle_events()?;
        }
        Ok(())
    }

    fn draw(&self, frame: &mut Frame<'_>, terminal_state: TerminalState) {
        ui(frame, self, terminal_state);
    }

    fn handle_events(&mut self) -> Result<()> {
        match event::read()? {
            Event::Key(key_event) if key_event.kind == KeyEventKind::Press => {
                self.handle_key_event(key_event)
            }
            _ => {}
        };
        Ok(())
    }

    fn handle_key_event(&mut self, key_event: KeyEvent) {
        match key_event.code {
            KeyCode::Char('q') => self.exit(),
            KeyCode::Char('1') => self.select_accounts_widget(),
            KeyCode::Char('2') => self.select_folders_widget(),
            KeyCode::Char('3') => self.select_messages_widget(),
            _ => {}
        }
    }

    fn select_accounts_widget(&mut self) {
        self.selected_widget = SelectedWidget::Accounts
    }

    fn select_folders_widget(&mut self) {
        self.selected_widget = SelectedWidget::Folders
    }

    fn select_messages_widget(&mut self) {
        self.selected_widget = SelectedWidget::Messages
    }

    fn exit(&mut self) {
        self.exit = true;
    }
}

pub fn ui(frame: &mut Frame<'_>, app: &App, terminal_state: TerminalState) {
    let layout_main = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Ratio(8, 9), Constraint::Length(2)])
        .split(frame.area());

    let layout_working_area = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Ratio(1, 4), Constraint::Ratio(3, 4)])
        .split(layout_main[0]);

    let layout_left_tower = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Ratio(2, 9), Constraint::Ratio(7, 9)])
        .split(layout_working_area[0]);

    let layout_rigth_tower = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Ratio(8, 9), Constraint::Ratio(1, 9)])
        .split(layout_working_area[1]);

    let status_widget = Block::bordered();
    let accounts_block =
        Block::bordered()
            .title("[1] Accounts")
            .border_style(match app.selected_widget {
                SelectedWidget::Accounts => Style::new().green(),
                _ => Style::default(),
            });

    let accounts_list = List::new(
        terminal_state
            .accounts
            .iter()
            .map(|a| a.login.to_string())
            .collect::<Vec<String>>(),
    )
    .block(accounts_block);

    let folders_block =
        Block::bordered()
            .title("[2] Folders")
            .border_style(match app.selected_widget {
                SelectedWidget::Folders => Style::new().green(),
                _ => Style::default(),
            });

    let messages_block =
        Block::bordered()
            .title("[3] Messages")
            .border_style(match app.selected_widget {
                SelectedWidget::Messages => Style::new().green(),
                _ => Style::default(),
            });

    let ads_block = Block::bordered().title("Advertisement");

    frame.render_widget(accounts_list, layout_left_tower[0]);
    frame.render_widget(folders_block, layout_left_tower[1]);
    frame.render_widget(messages_block, layout_rigth_tower[0]);
    frame.render_widget(ads_block, layout_rigth_tower[1]);
    frame.render_widget(status_widget, layout_main[1]);
}
