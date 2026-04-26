use ratatui::layout::{Constraint, Direction, Layout as RLayout, Rect};
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span, Text};
use ratatui::widgets::{Block, Borders, List, ListItem, Paragraph, Wrap};
use ratatui::Frame;

use crate::app::{App, ButtonFocus, Focus, Severity};

pub(crate) fn ui(frame: &mut Frame, app: &mut App) {
    let area = frame.area();

    let outer = RLayout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Min(0), Constraint::Length(3)])
        .split(area);

    let main_chunks = RLayout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(40), Constraint::Percentage(60)])
        .split(outer[0]);

    let left_chunks = RLayout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Min(0), Constraint::Length(3)])
        .split(main_chunks[0]);

    let button_chunks = RLayout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
        .split(left_chunks[1]);

    let right_chunks = RLayout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
        .split(main_chunks[1]);

    app.layout.list_area = left_chunks[0];
    app.layout.apply_area = button_chunks[0];
    app.layout.quit_area = button_chunks[1];
    app.layout.preview_area = right_chunks[0];
    app.layout.input_area = right_chunks[1];

    let input_inner_width = right_chunks[1].width.saturating_sub(2) as usize;
    app.ensure_cursor_visible(input_inner_width);

    render_app_list(frame, app, left_chunks[0]);
    render_buttons(frame, app, button_chunks[0], button_chunks[1]);
    render_preview(frame, app, right_chunks[0]);
    render_input(frame, app, right_chunks[1]);
    render_notifications(frame, app, outer[1]);
}

fn render_app_list(frame: &mut Frame, app: &mut App, area: Rect) {
    let focused = app.focus == Focus::AppList;
    let border_style = if focused {
        Style::default().fg(Color::Cyan)
    } else {
        Style::default()
    };

    let items: Vec<ListItem> = std::iter::once(("All apps", app.check_states[0]))
        .chain(
            app.app_names
                .iter()
                .enumerate()
                .map(|(i, n)| (n.as_str(), app.check_states[i + 1])),
        )
        .map(|(label, checked)| {
            let marker = if checked { "☑" } else { "☐" };
            ListItem::new(Line::from(vec![Span::raw(format!("{marker} ")), Span::raw(label)]))
        })
        .collect();

    let list = List::new(items)
        .block(
            Block::default()
                .title(" JetBrains Apps ")
                .borders(Borders::ALL)
                .border_style(border_style),
        )
        .highlight_style(
            Style::default()
                .bg(Color::DarkGray)
                .add_modifier(Modifier::BOLD),
        );

    frame.render_stateful_widget(list, area, &mut app.list_state);
}

fn render_buttons(frame: &mut Frame, app: &App, apply_rect: Rect, quit_rect: Rect) {
    let focused = app.focus == Focus::Buttons;
    let apply_active = focused && app.button_focus == ButtonFocus::Apply;
    let quit_active = focused && app.button_focus == ButtonFocus::Quit;

    let apply_style = if apply_active {
        Style::default().fg(Color::Black).bg(Color::Green)
    } else {
        Style::default().fg(Color::Green)
    };
    let quit_style = if quit_active {
        Style::default().fg(Color::Black).bg(Color::Red)
    } else {
        Style::default().fg(Color::Red)
    };

    let apply = Paragraph::new(Text::from(Line::from(vec![Span::styled(
        " ✔ Apply",
        apply_style,
    )])))
    .block(Block::default().borders(Borders::ALL).border_style(if apply_active {
        Style::default().fg(Color::Green)
    } else {
        Style::default()
    }));
    let quit = Paragraph::new(Text::from(Line::from(vec![Span::styled(" ✘ Quit", quit_style)])))
        .block(Block::default().borders(Borders::ALL).border_style(if quit_active {
            Style::default().fg(Color::Red)
        } else {
            Style::default()
        }));

    frame.render_widget(apply, apply_rect);
    frame.render_widget(quit, quit_rect);
}

fn render_preview(frame: &mut Frame, app: &App, area: Rect) {
    let text = app.preview_text();
    let preview = Paragraph::new(text)
        .block(Block::default().title(" Preview ").borders(Borders::ALL))
        .wrap(Wrap { trim: false })
        .scroll((app.preview_scroll, 0));
    frame.render_widget(preview, area);
}

fn render_input(frame: &mut Frame, app: &App, area: Rect) {
    let focused = app.focus == Focus::Input;

    let border_style = if focused {
        Style::default().fg(Color::Yellow)
    } else {
        Style::default()
    };

    let cursor_style = Style::default().fg(Color::Black).bg(Color::Yellow);
    let scroll = app.input_scroll;

    let text_lines: Vec<Line> = app
        .lines
        .iter()
        .enumerate()
        .map(|(row_idx, line)| {
            let visible: &str = if scroll < line.len() { &line[scroll..] } else { "" };

            if focused && row_idx == app.cursor_row {
                let cur = app.cursor_col.saturating_sub(scroll);
                let before = if cur <= visible.len() { &visible[..cur] } else { visible };
                let cursor_char = visible[cur..].chars().next().unwrap_or(' ');
                let after_off = cur + cursor_char.len_utf8().min(visible.len().saturating_sub(cur));
                let after = if cur < visible.len() { &visible[after_off..] } else { "" };

                Line::from(vec![
                    Span::raw(before),
                    Span::styled(cursor_char.to_string(), cursor_style),
                    Span::raw(after),
                ])
            } else {
                Line::from(Span::raw(visible))
            }
        })
        .collect();

    let hint = if focused { "" } else { " (click or Tab to edit)" };
    let title = format!(" VM Options{hint} ");

    let widget = Paragraph::new(Text::from(text_lines))
        .block(Block::default().title(title).borders(Borders::ALL).border_style(border_style));

    frame.render_widget(widget, area);
}

fn render_notifications(frame: &mut Frame, app: &App, area: Rect) {
    let msgs: Vec<Span> = app
        .notifications
        .iter()
        .rev()
        .take(3)
        .map(|n| {
            let color = match n.severity {
                Severity::Info => Color::Green,
                Severity::Warning => Color::Yellow,
                Severity::Error => Color::Red,
            };
            Span::styled(format!(" {} ", n.message), Style::default().fg(color))
        })
        .collect();

    let hint = Span::styled(
        " [Tab] Switch  [Space/Enter] Toggle  [↑↓←→] Navigate  [Esc/click] Exit editor  [C-c] Quit ",
        Style::default().fg(Color::DarkGray),
    );

    let line = if msgs.is_empty() {
        Line::from(vec![hint])
    } else {
        let mut parts = msgs;
        parts.push(Span::raw(" │ "));
        parts.push(hint);
        Line::from(parts)
    };

    let bar = Paragraph::new(line).block(Block::default().borders(Borders::TOP));
    frame.render_widget(bar, area);
}


