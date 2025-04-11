use ratatui::{
    layout::{Constraint, Direction, Layout},
    style::{Style, Stylize},
    widgets::{Block, List},
    Frame,
};

use crate::{
    app::{App, SelectedWidget},
    state::TerminalState,
};

pub fn ui(frame: &mut Frame<'_>, app: &mut App, terminal_state: TerminalState) {
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
                SelectedWidget::Accounts => Style::new().green().bold(),
                _ => Style::default(),
            });

    let accounts_list = List::new(
        terminal_state
            .accounts
            .iter()
            .map(|a| a.login.to_string())
            .collect::<Vec<String>>(),
    )
    .block(accounts_block)
    .highlight_style(Style::new().black().bg(ratatui::style::Color::Gray));

    let folders_block =
        Block::bordered()
            .title("[2] Folders")
            .border_style(match app.selected_widget {
                SelectedWidget::Folders => Style::new().green().bold(),
                _ => Style::default(),
            });

    let messages_block =
        Block::bordered()
            .title("[3] Messages")
            .border_style(match app.selected_widget {
                SelectedWidget::Messages => Style::new().green().bold(),
                _ => Style::default(),
            });

    let ads_block = Block::bordered().title("Advertisement");

    frame.render_stateful_widget(
        accounts_list,
        layout_left_tower[0],
        &mut app.accounts_list_state,
    );
    frame.render_widget(folders_block, layout_left_tower[1]);
    frame.render_widget(messages_block, layout_rigth_tower[0]);
    frame.render_widget(ads_block, layout_rigth_tower[1]);
    frame.render_widget(status_widget, layout_main[1]);
}
