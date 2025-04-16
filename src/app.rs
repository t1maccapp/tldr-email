use crate::{
    actions::Actions,
    state::{State, ViewState},
    ui::ui,
};
use anyhow::Result;
use crossterm::event::{self, Event, KeyCode, KeyEvent, KeyEventKind};
use ratatui::{
    widgets::{ListState, TableState},
    DefaultTerminal, Frame,
};
use std::{sync::Arc, time::Duration};
use tokio::sync::mpsc::UnboundedSender;

#[derive(Debug, Default, Eq, Hash, PartialEq)]
pub enum SelectedWidget {
    #[default]
    Accounts,
    Folders,
    Messages,
    Message,
    Send,
}

#[derive(Debug, Default, Eq, Hash, PartialEq)]
pub enum SelectedSendWidget {
    #[default]
    To,
    Subject,
    Text,
}

#[derive(Debug, Default)]
pub struct App {
    pub selected_widget: SelectedWidget,
    pub selected_send_widget: SelectedSendWidget,

    pub accounts_list_state: ListState,
    pub accounts_list_selected: Option<usize>,

    pub folders_list_state: ListState,
    pub folders_list_selected: Option<usize>,

    pub messages_table_state: TableState,
    pub messages_table_selected: Option<usize>,
    pub messages_table_page: usize,

    pub view_state: ViewState,

    pub should_mark_state_as_updating: bool,

    pub send_to: String,
    pub send_subject: String,
    pub send_text: String,

    pub exit: bool,
}

impl App {
    pub async fn run(
        &mut self,
        terminal: &mut DefaultTerminal,
        state: Arc<State>,
        actions_tx: UnboundedSender<Actions>,
    ) -> Result<()> {
        self.view_state = state.as_view_state(None).await;

        self.select_first_account(actions_tx.clone());

        while !self.exit {
            if let Some(selected_account_idx) = self.accounts_list_selected {
                let selected_account = self.view_state.accounts.get(selected_account_idx).cloned();

                self.select_first_folder_if_not_selected(actions_tx.clone());
                self.select_first_message_if_not_selected(actions_tx.clone());

                if !state.is_updating().await {
                    self.view_state = state.as_view_state(selected_account).await;
                }
            };

            terminal.draw(|frame| self.draw(frame))?;
            self.handle_events(actions_tx.clone())?;

            if self.should_mark_state_as_updating {
                *state.message.write().await = None; // TODO: workaround for now
                *state.is_updating.write().await = true;

                self.should_mark_state_as_updating = false;
            }
        }

        Ok(())
    }

    fn draw(&mut self, frame: &mut Frame<'_>) {
        ui(frame, self);
    }

    fn handle_events(&mut self, actions_tx: UnboundedSender<Actions>) -> Result<()> {
        if event::poll(Duration::from_millis(10))? {
            match event::read()? {
                Event::Key(key_event) if key_event.kind == KeyEventKind::Press => {
                    self.handle_key_event(key_event, actions_tx)
                }
                _ => {}
            };
        }

        Ok(())
    }

    fn handle_key_event(&mut self, key_event: KeyEvent, actions_tx: UnboundedSender<Actions>) {
        match self.selected_widget {
            SelectedWidget::Accounts => match key_event.code {
                KeyCode::Char('q') => self.exit(),
                KeyCode::Char('x') => Self::ads_remove(),
                KeyCode::Char('2') => self.select_folders_widget(),
                KeyCode::Char('3') => self.select_messages_widget(),
                KeyCode::Char('4') => self.select_message_widget(),
                KeyCode::Char('s') => self.select_send_widget(),
                KeyCode::Up | KeyCode::Char('k') => self.select_previous_account(actions_tx),
                KeyCode::Down | KeyCode::Char('j') => self.select_next_account(actions_tx),
                _ => {}
            },
            SelectedWidget::Folders => match key_event.code {
                KeyCode::Char('q') => self.exit(),
                KeyCode::Char('x') => Self::ads_remove(),
                KeyCode::Char('1') => self.select_accounts_widget(),
                KeyCode::Char('3') => self.select_messages_widget(),
                KeyCode::Char('4') => self.select_message_widget(),
                KeyCode::Char('s') => self.select_send_widget(),
                KeyCode::Up | KeyCode::Char('k') => self.select_previous_folder(actions_tx),
                KeyCode::Down | KeyCode::Char('j') => self.select_next_folder(actions_tx),
                _ => {}
            },
            SelectedWidget::Messages => match key_event.code {
                KeyCode::Char('q') => self.exit(),
                KeyCode::Char('x') => Self::ads_remove(),
                KeyCode::Char('1') => self.select_accounts_widget(),
                KeyCode::Char('2') => self.select_folders_widget(),
                KeyCode::Char('4') => self.select_message_widget(),
                KeyCode::Char('s') => self.select_send_widget(),
                KeyCode::Up | KeyCode::Char('k') => self.select_previous_message(actions_tx),
                KeyCode::Down | KeyCode::Char('j') => self.select_next_message(actions_tx),
                KeyCode::Left | KeyCode::Char('p') => self.select_previous_message_page(actions_tx),
                KeyCode::Right | KeyCode::Char('n') => self.select_next_message_page(actions_tx),
                _ => {}
            },
            SelectedWidget::Message => match key_event.code {
                KeyCode::Char('q') => self.exit(),
                KeyCode::Char('x') => Self::ads_remove(),
                KeyCode::Char('1') => self.select_accounts_widget(),
                KeyCode::Char('2') => self.select_folders_widget(),
                KeyCode::Char('3') => self.select_messages_widget(),
                KeyCode::Char('s') => self.select_send_widget(),
                _ => {}
            },
            SelectedWidget::Send => match key_event.code {
                KeyCode::Esc => {
                    // TODO: refactor
                    self.send_to = String::default();
                    self.send_text = String::default();
                    self.send_subject = String::default();
                    self.select_accounts_widget()
                }
                KeyCode::Tab => self.select_next_send_widget(),
                KeyCode::BackTab => self.select_previous_send_widget(),
                KeyCode::Char(value) => match self.selected_send_widget {
                    SelectedSendWidget::To => self.send_to.push(value),
                    SelectedSendWidget::Subject => self.send_subject.push(value),
                    SelectedSendWidget::Text => self.send_text.push(value),
                },
                KeyCode::Backspace => match self.selected_send_widget {
                    SelectedSendWidget::To => {
                        if self.send_to.len() > 0 {
                            let _ = self.send_to.pop();
                        }
                    }
                    SelectedSendWidget::Subject => {
                        if self.send_subject.len() > 0 {
                            let _ = self.send_subject.pop();
                        }
                    }
                    SelectedSendWidget::Text => {
                        if self.send_text.len() > 0 {
                            let _ = self.send_text.pop();
                        }
                    }
                },

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

    fn select_message_widget(&mut self) {
        self.selected_widget = SelectedWidget::Message
    }

    fn select_send_widget(&mut self) {
        self.selected_widget = SelectedWidget::Send
    }

    fn select_previous_send_widget(&mut self) {
        match self.selected_send_widget {
            SelectedSendWidget::To => self.selected_send_widget = SelectedSendWidget::Text,
            SelectedSendWidget::Subject => self.selected_send_widget = SelectedSendWidget::To,
            SelectedSendWidget::Text => self.selected_send_widget = SelectedSendWidget::Subject,
        }
    }

    fn select_next_send_widget(&mut self) {
        match self.selected_send_widget {
            SelectedSendWidget::To => self.selected_send_widget = SelectedSendWidget::Subject,
            SelectedSendWidget::Subject => self.selected_send_widget = SelectedSendWidget::Text,
            SelectedSendWidget::Text => self.selected_send_widget = SelectedSendWidget::To,
        }
    }

    fn exit(&mut self) {
        self.exit = true;
    }

    fn select_first_account(&mut self, actions_tx: UnboundedSender<Actions>) {
        self.accounts_list_selected = Some(0);
        self.accounts_list_state.select(self.accounts_list_selected);

        self.clear_folders();
        self.clear_messages();
        self.clear_message();

        if let Some(selected_account_idx) = self.accounts_list_selected {
            let selected_account = self.view_state.accounts.get(selected_account_idx).cloned();

            if let Some(login) = selected_account {
                self.should_mark_state_as_updating = true;
                let _ = actions_tx.send(Actions::ListFolders { login });
            }
        };
    }

    fn select_previous_account(&mut self, actions_tx: UnboundedSender<Actions>) {
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

        self.clear_folders();
        self.clear_messages();
        self.clear_message();

        let selected_account = self.view_state.accounts.get(previous_account_idx).cloned();

        if let Some(login) = selected_account {
            self.should_mark_state_as_updating = true;
            let _ = actions_tx.send(Actions::ListFolders { login });
        }
    }

    fn select_next_account(&mut self, actions_tx: UnboundedSender<Actions>) {
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

        self.clear_folders();
        self.clear_messages();
        self.clear_message();

        let selected_account = self.view_state.accounts.get(next_account_idx).cloned();

        if let Some(login) = selected_account {
            self.should_mark_state_as_updating = true;
            let _ = actions_tx.send(Actions::ListFolders { login });
        }
    }

    fn select_first_folder_if_not_selected(&mut self, actions_tx: UnboundedSender<Actions>) {
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

        self.clear_messages();
        self.clear_message();

        self.should_mark_state_as_updating = true;
        let _ = actions_tx.send(Actions::ListEnvelopes {
            login,
            folder,
            page: 0,
        });
    }

    fn select_previous_folder(&mut self, actions_tx: UnboundedSender<Actions>) {
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

        self.clear_messages();
        self.clear_message();

        self.should_mark_state_as_updating = true;
        let _ = actions_tx.send(Actions::ListEnvelopes {
            login,
            folder,
            page: 0,
        });
    }

    fn select_next_folder(&mut self, actions_tx: UnboundedSender<Actions>) {
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

        self.clear_messages();
        self.clear_message();

        self.should_mark_state_as_updating = true;
        let _ = actions_tx.send(Actions::ListEnvelopes {
            login,
            folder,
            page: 0,
        });
    }

    fn select_first_message_if_not_selected(&mut self, actions_tx: UnboundedSender<Actions>) {
        if self.messages_table_selected.is_none() && self.view_state.messages.is_some() {
            self.messages_table_selected = Some(0);
            self.messages_table_state
                .select(self.messages_table_selected);
        } else {
            return;
        }

        self.clear_message();

        let Some(selected_account_idx) = self.accounts_list_selected else {
            return;
        };

        let Some(login) = self.view_state.accounts.get(selected_account_idx).cloned() else {
            return;
        };

        let Some(folders) = &self.view_state.folders else {
            return;
        };

        let Some(selected_folder_idx) = self.folders_list_selected else {
            return;
        };

        let Some(folder) = folders.get(selected_folder_idx).cloned() else {
            return;
        };

        let Some(messages) = &self.view_state.messages else {
            return;
        };

        let Some(selected_message_idx) = self.messages_table_selected else {
            return;
        };

        let Some(envelope) = messages.get(selected_message_idx) else {
            return;
        };

        self.should_mark_state_as_updating = true;
        let _ = actions_tx.send(Actions::GetMessage {
            login,
            folder,
            id: envelope.id.clone(),
        });
    }

    fn select_previous_message(&mut self, actions_tx: UnboundedSender<Actions>) {
        let Some(selected_message_idx) = self.messages_table_selected else {
            return;
        };

        let Some(messages) = &self.view_state.messages else {
            return;
        };

        if messages.len() == 0 {
            return;
        }

        let previous_message_idx = if selected_message_idx == 0 {
            messages.len() - 1
        } else {
            selected_message_idx - 1
        };

        self.messages_table_selected = Some(previous_message_idx);
        self.messages_table_state
            .select(self.messages_table_selected);

        self.clear_message();

        let Some(selected_account_idx) = self.accounts_list_selected else {
            return;
        };

        let Some(login) = self.view_state.accounts.get(selected_account_idx).cloned() else {
            return;
        };

        let Some(folders) = &self.view_state.folders else {
            return;
        };

        let Some(selected_folder_idx) = self.folders_list_selected else {
            return;
        };

        let Some(folder) = folders.get(selected_folder_idx).cloned() else {
            return;
        };

        let Some(messages) = &self.view_state.messages else {
            return;
        };

        let Some(selected_message_idx) = self.messages_table_selected else {
            return;
        };

        let Some(envelope) = messages.get(selected_message_idx) else {
            return;
        };

        self.should_mark_state_as_updating = true;
        let _ = actions_tx.send(Actions::GetMessage {
            login,
            folder,
            id: envelope.id.clone(),
        });
    }

    fn select_next_message(&mut self, actions_tx: UnboundedSender<Actions>) {
        let Some(selected_message_idx) = self.messages_table_selected else {
            return;
        };

        let Some(messages) = &self.view_state.messages else {
            return;
        };

        if messages.len() == 0 {
            return;
        }

        let next_message_idx = if selected_message_idx == messages.len() - 1 {
            0
        } else {
            selected_message_idx + 1
        };

        self.messages_table_selected = Some(next_message_idx);
        self.messages_table_state
            .select(self.messages_table_selected);

        self.clear_message();

        let Some(selected_account_idx) = self.accounts_list_selected else {
            return;
        };

        let Some(login) = self.view_state.accounts.get(selected_account_idx).cloned() else {
            return;
        };

        let Some(folders) = &self.view_state.folders else {
            return;
        };

        let Some(selected_folder_idx) = self.folders_list_selected else {
            return;
        };

        let Some(folder) = folders.get(selected_folder_idx).cloned() else {
            return;
        };

        let Some(messages) = &self.view_state.messages else {
            return;
        };

        let Some(selected_message_idx) = self.messages_table_selected else {
            return;
        };

        let Some(envelope) = messages.get(selected_message_idx) else {
            return;
        };

        self.should_mark_state_as_updating = true;
        let _ = actions_tx.send(Actions::GetMessage {
            login,
            folder,
            id: envelope.id.clone(),
        });
    }

    fn select_previous_message_page(&mut self, actions_tx: UnboundedSender<Actions>) {
        let Some(selected_account_idx) = self.accounts_list_selected else {
            return;
        };

        let Some(login) = self.view_state.accounts.get(selected_account_idx).cloned() else {
            return;
        };

        let Some(folders) = &self.view_state.folders else {
            return;
        };

        let Some(selected_folder_idx) = self.folders_list_selected else {
            return;
        };

        let Some(folder) = folders.get(selected_folder_idx).cloned() else {
            return;
        };

        if self.messages_table_page == 0 {
            return;
        } else {
            self.messages_table_page -= 1;
        };

        self.messages_table_state = TableState::default();
        self.messages_table_selected = None;
        self.view_state.messages = None;

        self.should_mark_state_as_updating = true;
        let _ = actions_tx.send(Actions::ListEnvelopes {
            login,
            folder,
            page: self.messages_table_page,
        });
    }

    fn select_next_message_page(&mut self, actions_tx: UnboundedSender<Actions>) {
        let Some(selected_account_idx) = self.accounts_list_selected else {
            return;
        };

        let Some(login) = self.view_state.accounts.get(selected_account_idx).cloned() else {
            return;
        };

        let Some(folders) = &self.view_state.folders else {
            return;
        };

        let Some(selected_folder_idx) = self.folders_list_selected else {
            return;
        };

        let Some(folder) = folders.get(selected_folder_idx).cloned() else {
            return;
        };

        let Some(messages) = &self.view_state.messages else {
            return;
        };

        if messages.len() == 10 {
            self.messages_table_state = TableState::default();
            self.messages_table_selected = None;
            self.view_state.messages = None;
            self.messages_table_page += 1;
        } else {
            return;
        };

        self.should_mark_state_as_updating = true;
        let _ = actions_tx.send(Actions::ListEnvelopes {
            login,
            folder,
            page: self.messages_table_page,
        });
    }

    fn clear_folders(&mut self) {
        self.folders_list_state = ListState::default();
        self.folders_list_selected = None;
        self.view_state.folders = None;
    }

    fn clear_messages(&mut self) {
        self.messages_table_state = TableState::default();
        self.messages_table_selected = None;
        self.messages_table_page = 0;
        self.view_state.messages = None;
    }

    fn clear_message(&mut self) {
        self.view_state.message = None;
    }

    fn ads_remove() {
        let _ = webbrowser::open("https://www.youtube.com/watch?v=dQw4w9WgXcQ");
    }
}
