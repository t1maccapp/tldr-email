use ratatui::{
    layout::{Constraint, Direction, Layout, Margin},
    style::{Style, Stylize},
    widgets::{Block, List},
    Frame,
};

use crate::app::{App, SelectedWidget};

pub fn ui(frame: &mut Frame<'_>, app: &mut App) {
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

    let accounts_list = List::new(app.view_state.accounts.clone())
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

    if let Some(folders) = &app.view_state.folders {
        let folders_list = List::new(folders.clone())
            .block(folders_block)
            .highlight_style(Style::new().black().bg(ratatui::style::Color::Gray));

        frame.render_stateful_widget(
            folders_list,
            layout_left_tower[1],
            &mut app.folders_list_state,
        );
    } else {
        frame.render_widget(folders_block, layout_left_tower[1]);

        frame.render_widget(
            throbber_widgets_tui::Throbber::default(),
            layout_left_tower[1].inner(Margin::new(1, 1)),
        );
    };

    if let Some(messages) = &app.view_state.messages {
        let messages_list = List::new(messages.clone())
            .block(messages_block)
            .highlight_style(Style::new().black().bg(ratatui::style::Color::Gray));

        frame.render_stateful_widget(
            messages_list,
            layout_rigth_tower[0],
            &mut app.messages_list_state,
        );
    } else {
        frame.render_widget(messages_block, layout_rigth_tower[0]);

        frame.render_widget(
            throbber_widgets_tui::Throbber::default(),
            layout_rigth_tower[0].inner(Margin::new(1, 1)),
        );
    };

    frame.render_widget(ads_block, layout_rigth_tower[1]);
    frame.render_widget(status_widget, layout_main[1]);
}
