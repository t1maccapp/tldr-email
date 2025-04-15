use crate::{
    actions::Actions,
    state::{State, ViewState},
    ui::ui,
};
use anyhow::Result;
use crossterm::event::{self, Event, KeyCode, KeyEvent, KeyEventKind};
use ratatui::{widgets::ListState, DefaultTerminal, Frame};
use std::{sync::Arc, time::Duration};
use tokio::sync::mpsc::UnboundedSender;

#[derive(Debug, Default, Eq, Hash, PartialEq)]
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
    pub accounts_list_selected: Option<usize>,
    pub folders_list_state: ListState,
    pub folders_list_selected: Option<usize>,
    pub messages_list_state: ListState,
    pub messages_list_selected: Option<usize>,

    pub view_state: ViewState,

    pub exit: bool,
}

impl App {
    pub async fn run(
        &mut self,
        terminal: &mut DefaultTerminal,
        state: Arc<State>,
        tx: UnboundedSender<Actions>,
    ) -> Result<()> {
        self.view_state = state.as_view_state(None).await;

        self.select_first_account(tx.clone());

        while !self.exit {
            if let Some(selected_account_idx) = self.accounts_list_selected {
                let selected_account = self.view_state.accounts.get(selected_account_idx).cloned();

                self.select_first_folder_if_not_selected(tx.clone());

                self.view_state = state.as_view_state(selected_account).await;
            };

            terminal.draw(|frame| self.draw(frame))?;
            self.handle_events(tx.clone())?;
        }

        Ok(())
    }

    fn draw(&mut self, frame: &mut Frame<'_>) {
        ui(frame, self);
    }

    fn handle_events(&mut self, tx: UnboundedSender<Actions>) -> Result<()> {
        if event::poll(Duration::from_millis(10))? {
            match event::read()? {
                Event::Key(key_event) if key_event.kind == KeyEventKind::Press => {
                    self.handle_key_event(key_event, tx)
                }
                _ => {}
            };
        }

        Ok(())
    }

    fn handle_key_event(&mut self, key_event: KeyEvent, tx: UnboundedSender<Actions>) {
        match self.selected_widget {
            SelectedWidget::Accounts => match key_event.code {
                KeyCode::Char('q') => self.exit(),
                KeyCode::Char('2') => self.select_folders_widget(),
                KeyCode::Char('3') => self.select_messages_widget(),
                KeyCode::Up | KeyCode::Char('k') => self.select_previous_account(tx),
                KeyCode::Down | KeyCode::Char('j') => self.select_next_account(tx),
                _ => {}
            },
            SelectedWidget::Folders => match key_event.code {
                KeyCode::Char('q') => self.exit(),
                KeyCode::Char('1') => self.select_accounts_widget(),
                KeyCode::Char('3') => self.select_messages_widget(),
                KeyCode::Up | KeyCode::Char('k') => self.select_previous_folder(tx),
                KeyCode::Down | KeyCode::Char('j') => self.select_next_folder(tx),
                _ => {}
            },
            SelectedWidget::Messages => match key_event.code {
                KeyCode::Char('q') => self.exit(),
                KeyCode::Char('1') => self.select_accounts_widget(),
                KeyCode::Char('2') => self.select_folders_widget(),
                _ => {}
            },
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

    fn select_first_account(&mut self, tx: UnboundedSender<Actions>) {
        self.accounts_list_selected = Some(0);
        self.accounts_list_state.select(self.accounts_list_selected);

        self.folders_list_state = ListState::default();
        self.folders_list_selected = None;
        self.view_state.folders = None;
        self.messages_list_state = ListState::default();
        self.messages_list_selected = None;
        self.view_state.messages = None;

        if let Some(selected_account_idx) = self.accounts_list_selected {
            let selected_account = self.view_state.accounts.get(selected_account_idx).cloned();

            if let Some(login) = selected_account {
                let _ = tx.send(Actions::ListFolders { login });
            }
        };
    }

    fn select_previous_account(&mut self, tx: UnboundedSender<Actions>) {
        let Some(selected_account_idx) = self.accounts_list_selected else {
            return;
        };

        let previous_account_idx = if selected_account_idx == 0 {
            self.view_state.accounts.len() - 1
        } else {
            selected_account_idx - 1
        };
        self.accounts_list_selected = Some(previous_account_idx);
        self.accounts_list_state.select(self.accounts_list_selected);

        self.folders_list_state = ListState::default();
        self.folders_list_selected = None;
        self.view_state.folders = None;
        self.messages_list_state = ListState::default();
        self.messages_list_selected = None;
        self.view_state.messages = None;

        let selected_account = self.view_state.accounts.get(previous_account_idx).cloned();

        if let Some(login) = selected_account {
            let _ = tx.send(Actions::ListFolders { login });
        }
    }

    fn select_next_account(&mut self, tx: UnboundedSender<Actions>) {
        let Some(selected_account_idx) = self.accounts_list_selected else {
            return;
        };

        let next_account_idx = if selected_account_idx == self.view_state.accounts.len() - 1 {
            0
        } else {
            selected_account_idx + 1
        };
        self.accounts_list_selected = Some(next_account_idx);
        self.accounts_list_state.select(self.accounts_list_selected);

        self.folders_list_state = ListState::default();
        self.folders_list_selected = None;
        self.view_state.folders = None;
        self.messages_list_state = ListState::default();
        self.messages_list_selected = None;
        self.view_state.messages = None;

        let selected_account = self.view_state.accounts.get(next_account_idx).cloned();

        if let Some(login) = selected_account {
            let _ = tx.send(Actions::ListFolders { login });
        }
    }

    fn select_first_folder_if_not_selected(&mut self, tx: UnboundedSender<Actions>) {
        if self.folders_list_selected.is_none() && self.view_state.folders.is_some() {
            self.folders_list_selected = Some(0);
            self.folders_list_state.select(self.folders_list_selected);
        } else {
            return;
        }

        let Some(selected_account_idx) = self.accounts_list_selected else {
            return;
        };

        let Some(login) = self.view_state.accounts.get(selected_account_idx).cloned() else {
            return;
        };

        let Some(selected_folder_idx) = self.folders_list_selected else {
            return;
        };

        let Some(folders) = &self.view_state.folders else {
            return;
        };

        let Some(folder) = folders.get(selected_folder_idx).cloned() else {
            return;
        };

        self.messages_list_state = ListState::default();
        self.messages_list_selected = None;
        self.view_state.messages = None;

        let _ = tx.send(Actions::ListEnvelopes {
            login,
            folder,
            page: 0,
        });
    }

    fn select_previous_folder(&mut self, tx: UnboundedSender<Actions>) {
        let Some(selected_account_idx) = self.accounts_list_selected else {
            return;
        };

        let Some(login) = self.view_state.accounts.get(selected_account_idx).cloned() else {
            return;
        };

        let Some(selected_folder_idx) = self.folders_list_selected else {
            return;
        };

        let Some(folders) = &self.view_state.folders else {
            return;
        };

        let previous_folder_idx = if selected_folder_idx == 0 {
            folders.len() - 1
        } else {
            selected_folder_idx - 1
        };

        let Some(folder) = folders.get(previous_folder_idx).cloned() else {
            return;
        };

        self.folders_list_selected = Some(previous_folder_idx);
        self.folders_list_state.select(self.folders_list_selected);

        self.messages_list_state = ListState::default();
        self.messages_list_selected = None;
        self.view_state.messages = None;

        let _ = tx.send(Actions::ListEnvelopes {
            login,
            folder,
            page: 0,
        });
    }

    fn select_next_folder(&mut self, tx: UnboundedSender<Actions>) {
        let Some(selected_account_idx) = self.accounts_list_selected else {
            return;
        };

        let Some(login) = self.view_state.accounts.get(selected_account_idx).cloned() else {
            return;
        };

        let Some(selected_folder_idx) = self.folders_list_selected else {
            return;
        };

        let Some(folders) = &self.view_state.folders else {
            return;
        };

        let next_folder_idx = if selected_folder_idx == folders.len() - 1 {
            0
        } else {
            selected_folder_idx + 1
        };

        let Some(folder) = folders.get(next_folder_idx).cloned() else {
            return;
        };

        self.folders_list_selected = Some(next_folder_idx);
        self.folders_list_state.select(self.folders_list_selected);

        self.messages_list_state = ListState::default();
        self.messages_list_selected = None;
        self.view_state.messages = None;

        let _ = tx.send(Actions::ListEnvelopes {
            login,
            folder,
            page: 0,
        });
    }
}
