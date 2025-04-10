use std::sync::Arc;

use anyhow::Result;
use crossterm::event::{self, Event, KeyCode, KeyEvent, KeyEventKind};
use ratatui::{
    layout::{Constraint, Direction, Layout},
    style::{Style, Stylize},
    widgets::Block,
    DefaultTerminal, Frame,
};

#[tokio::main]
async fn main() -> Result<()> {
    let mut terminal = ratatui::init();

    let state = Arc::new(State::default());

    let app_result = App::default().run(&mut terminal, state.clone());

    ratatui::restore();

    app_result
}

#[derive(Debug, Default)]
pub struct State {}

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
    exit: bool,
}

impl App {
    pub fn run(&mut self, terminal: &mut DefaultTerminal, state: Arc<State>) -> Result<()> {
        while !self.exit {
            terminal.draw(|frame| self.draw(frame))?;
            self.handle_events()?;
        }
        Ok(())
    }

    fn draw(&self, frame: &mut Frame) {
        ui(frame, self)
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

pub fn ui(frame: &mut Frame, app: &App) {
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
        .constraints([Constraint::Ratio(1, 9), Constraint::Ratio(8, 9)])
        .split(layout_working_area[0]);

    let layout_rigth_tower = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Ratio(8, 9), Constraint::Ratio(1, 9)])
        .split(layout_working_area[1]);

    let status_widget = Block::bordered();
    let mut accounts_block = Block::bordered().title("[1] Accounts");
    let mut folders_block = Block::bordered().title("[2] Folders");
    let mut messages_block = Block::bordered().title("[3] Messages");
    let ads_block = Block::bordered().title("Advertisement");

    match app.selected_widget {
        SelectedWidget::Accounts => {
            accounts_block = accounts_block.border_style(Style::new().green()).bold()
        }
        SelectedWidget::Folders => {
            folders_block = folders_block.border_style(Style::new().green()).bold()
        }
        SelectedWidget::Messages => {
            messages_block = messages_block.border_style(Style::new().green()).bold()
        }
    };

    frame.render_widget(accounts_block, layout_left_tower[0]);
    frame.render_widget(folders_block, layout_left_tower[1]);
    frame.render_widget(messages_block, layout_rigth_tower[0]);
    frame.render_widget(ads_block, layout_rigth_tower[1]);
    frame.render_widget(status_widget, layout_main[1]);
}
