use anyhow::Result;
use crossterm::event::{self, Event, KeyCode, KeyEvent, KeyEventKind};
use ratatui::{
    layout::{Constraint, Direction, Layout},
    style::Style,
    widgets::{Block, Borders},
    DefaultTerminal, Frame,
};

#[tokio::main]
async fn main() -> Result<()> {
    let mut terminal = ratatui::init();

    let app_result = App::default().run(&mut terminal);

    ratatui::restore();

    app_result
}

#[derive(Debug, Default)]
pub struct App {
    exit: bool,
}

impl App {
    pub fn run(&mut self, terminal: &mut DefaultTerminal) -> Result<()> {
        while !self.exit {
            terminal.draw(|frame| self.draw(frame))?;
            self.handle_events()?;
        }
        Ok(())
    }

    fn draw(&self, frame: &mut Frame) {
        ui(frame)
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
            _ => {}
        }
    }

    fn exit(&mut self) {
        self.exit = true;
    }
}

pub fn ui(frame: &mut Frame) {
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

    let accounts_block = Block::bordered().title("[1] Accounts");

    let folders_block = Block::bordered().title("[2] Folders");

    let messages_block = Block::bordered().title("[3] Messages");

    let ads_block = Block::bordered().title("[4] Ads");

    frame.render_widget(accounts_block, layout_left_tower[0]);

    frame.render_widget(folders_block, layout_left_tower[1]);

    frame.render_widget(messages_block, layout_rigth_tower[0]);

    frame.render_widget(ads_block, layout_rigth_tower[1]);

    frame.render_widget(status_widget, layout_main[1]);
}
