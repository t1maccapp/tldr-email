use crate::{
    state::{State, TerminalState},
    ui::ui,
};
use anyhow::Result;
use crossterm::event::{self, Event, KeyCode, KeyEvent, KeyEventKind};
use ratatui::{widgets::ListState, DefaultTerminal, Frame};
use std::sync::Arc;

#[derive(Debug, Default)]
pub enum SelectedWidget {
    #[default]
    Accounts,
    Folders,
    Messages,
}

#[derive(Debug, Default)]
pub struct App {
    pub selected_widget: SelectedWidget,
    pub accounts_list_state: ListState,
    pub exit: bool,
}

impl App {
    pub async fn run(&mut self, terminal: &mut DefaultTerminal, state: Arc<State>) -> Result<()> {
        self.select_first_account();

        while !self.exit {
            let terminal_state = state.as_terminal_state().await;

            terminal.draw(|frame| self.draw(frame, terminal_state))?;
            self.handle_events()?;
        }

        Ok(())
    }

    fn draw(&mut self, frame: &mut Frame<'_>, terminal_state: TerminalState) {
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

    fn select_first_account(&mut self) {
        self.accounts_list_state.select_first();
    }
}
