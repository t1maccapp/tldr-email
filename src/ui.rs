use email_address::EmailAddress;
use ratatui::{
    layout::{Constraint, Direction, Layout, Margin},
    style::{Style, Stylize},
    text::Line,
    widgets::{Block, Clear, List, Paragraph, Row, Table},
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
        .constraints([
            Constraint::Ratio(3, 9),
            Constraint::Ratio(5, 9),
            Constraint::Ratio(1, 9),
        ])
        .split(layout_working_area[1]);

    let send_to_is_valid = EmailAddress::is_valid(&app.send_to);

    let status_widget = Block::default().title(match app.selected_widget {
        SelectedWidget::Accounts => Line::from(vec![
            " Quit ".into(),
            "<q>".blue().bold(),
            "     ".into(),
            " previous acc ".into(),
            "<k> ".blue().bold(),
            "<Up>".blue().bold(),
            "     ".into(),
            " next acc ".into(),
            "<j> ".blue().bold(),
            "<Down>".blue().bold(),
            "     ".into(),
            " send new ".into(),
            "<s> ".blue().bold(),
        ])
        .centered(),
        SelectedWidget::Folders => Line::from(vec![
            " Quit ".into(),
            "<q>".blue().bold(),
            "     ".into(),
            " previous folder ".into(),
            "<k> ".blue().bold(),
            "<Up>".blue().bold(),
            "     ".into(),
            " next folder ".into(),
            "<j> ".blue().bold(),
            "<Down>".blue().bold(),
            "     ".into(),
            " send new ".into(),
            "<s> ".blue().bold(),
        ])
        .centered(),
        SelectedWidget::Messages => Line::from(vec![
            " Quit ".into(),
            "<q>".blue().bold(),
            "     ".into(),
            " previous message ".into(),
            "<k> ".blue().bold(),
            "<Up>".blue().bold(),
            "     ".into(),
            " next message ".into(),
            "<j> ".blue().bold(),
            "<Down>".blue().bold(),
            "     ".into(),
            " previous page ".into(),
            "<p> ".blue().bold(),
            "<Left>".blue().bold(),
            "     ".into(),
            " next page ".into(),
            "<n> ".blue().bold(),
            "<Right>".blue().bold(),
            "     ".into(),
            " send new ".into(),
            "<s> ".blue().bold(),
        ])
        .centered(),
        SelectedWidget::Message => Line::from(vec![
            " Quit ".into(),
            "<q>".blue().bold(),
            "     ".into(),
            " send new ".into(),
            "<s> ".blue().bold(),
        ]),
        SelectedWidget::Send => {
            let mut words = vec![
                " Close ".into(),
                "<Esc>".blue().bold(),
                "     ".into(),
                " Next input ".into(),
                "<Tab>".blue().bold(),
                "     ".into(),
                " Previous input ".into(),
                "<Shift + Tab>".blue().bold(),
            ];

            if send_to_is_valid {
                words.extend(vec![
                    "     ".into(),
                    " Send ".into(),
                    "<Enter>".blue().bold(),
                ]);
            }
            Line::from(words)
        }
    });
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

    let message_block =
        Block::bordered()
            .title("[4] Message")
            .border_style(match app.selected_widget {
                SelectedWidget::Message => Style::new().green().bold(),
                _ => Style::default(),
            });

    let ads_block = Paragraph::new(
        "Act now! Limited time offer! Buy this energy drink or miss out forever! Don't regret it!",
    )
    .block(
        Block::bordered()
            .title_bottom(Line::from("press <x> to hide ads").right_aligned())
            .title("Ads"),
    );

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
        let rows = messages
            .iter()
            .map(|e| {
                Row::new(vec![
                    e.id.clone(),
                    e.flags.to_string(),
                    e.subject.clone(),
                    e.from.to_string(),
                    e.date.to_string(),
                ])
            })
            .collect::<Vec<Row>>();
        let widths = [
            Constraint::Ratio(1, 18),
            Constraint::Ratio(2, 18),
            Constraint::Ratio(11, 18),
            Constraint::Ratio(2, 18),
            Constraint::Ratio(2, 18),
        ];
        let messages_table = Table::new(rows, widths)
            .header(Row::new(vec!["id", "flags", "subject", "from", "date"]))
            .block(messages_block)
            .footer(Row::new(vec![format!(
                "page: {}",
                app.messages_table_page.to_string()
            )]))
            .row_highlight_style(Style::new().black().bg(ratatui::style::Color::Gray));

        frame.render_stateful_widget(
            messages_table,
            layout_rigth_tower[0],
            &mut app.messages_table_state,
        );
    } else {
        frame.render_widget(messages_block, layout_rigth_tower[0]);

        frame.render_widget(
            throbber_widgets_tui::Throbber::default(),
            layout_rigth_tower[0].inner(Margin::new(1, 1)),
        );
    };

    if let Some(message) = &app.view_state.message {
        let message_p = if app.messages_table_selected.is_some() {
            Paragraph::new(message.clone()).block(message_block)
        } else {
            Paragraph::new("").block(message_block)
        };

        frame.render_widget(message_p, layout_rigth_tower[1]);
    } else {
        frame.render_widget(message_block, layout_rigth_tower[1]);

        frame.render_widget(
            throbber_widgets_tui::Throbber::default(),
            layout_rigth_tower[1].inner(Margin::new(1, 1)),
        );
    }

    frame.render_widget(ads_block, layout_rigth_tower[2]);
    frame.render_widget(status_widget, layout_main[1]);

    if app.selected_widget == SelectedWidget::Send {
        let Some(selected_account_idx) = app.accounts_list_selected else {
            return;
        };

        let selected_account = app.view_state.accounts.get(selected_account_idx).cloned();

        let Some(login) = selected_account else {
            return;
        };

        frame.render_widget(Clear, layout_main[0]);

        frame.render_widget(
            Block::bordered()
                .title(Line::from(format!("Send new message from: {}", login)).left_aligned()),
            layout_main[0],
        );

        let inner_area = layout_main[0].inner(Margin::new(1, 1));

        let inner_layout = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Ratio(1, 9),
                Constraint::Ratio(2, 9),
                Constraint::Ratio(6, 9),
            ])
            .split(inner_area);

        frame.render_widget(
            Paragraph::new(app.send_to.clone())
                .block(
                    Block::bordered()
                        .title("To")
                        .border_style(match app.selected_send_widget {
                            crate::app::SelectedSendWidget::To => Style::new().green().bold(),
                            _ => Style::default(),
                        }),
                )
                .style(if send_to_is_valid {
                    Style::default().green()
                } else {
                    Style::default().red()
                }),
            inner_layout[0],
        );

        frame.render_widget(
            Paragraph::new(app.send_subject.clone()).block(
                Block::bordered()
                    .title("Subject")
                    .border_style(match app.selected_send_widget {
                        crate::app::SelectedSendWidget::Subject => Style::new().green().bold(),
                        _ => Style::default(),
                    }),
            ),
            inner_layout[1],
        );

        frame.render_widget(
            Paragraph::new(app.send_text.clone()).block(
                Block::bordered()
                    .title("Text")
                    .border_style(match app.selected_send_widget {
                        crate::app::SelectedSendWidget::Text => Style::new().green().bold(),
                        _ => Style::default(),
                    }),
            ),
            inner_layout[2],
        );
    }
}
