use std::time::Duration;

use anyhow::Result;
use crossterm::event::{self, Event, KeyCode, KeyEventKind, KeyModifiers};
use ratatui::backend::Backend;
use ratatui::Terminal;

use crate::app::{App, ButtonFocus, Focus};
use crate::render::ui;

pub fn drain_events() {
    while event::poll(Duration::from_millis(0)).unwrap_or(false) {
        let _ = event::read();
    }
}

pub fn run_app<B: Backend>(terminal: &mut Terminal<B>, app: &mut App) -> Result<()>
where
    <B as Backend>::Error: Send,
    <B as Backend>::Error: Sync,
    <B as Backend>::Error: 'static,
{
    loop {
        app.prune_notifications();
        terminal.draw(|f| ui(f, app))?;

        if event::poll(Duration::from_millis(200))? {
            match event::read()? {
                Event::Key(key) => {
                    if key.kind == KeyEventKind::Release {
                        continue;
                    }
                    if key.code == KeyCode::Char('c')
                        && key.modifiers.contains(KeyModifiers::CONTROL)
                    {
                        return Ok(());
                    }

                    match &app.focus {
                        Focus::Input => match key.code {
                            KeyCode::Esc => app.focus = Focus::AppList,
                            KeyCode::Tab => app.focus = Focus::AppList,
                            KeyCode::Enter => {
                                if key.modifiers.contains(KeyModifiers::CONTROL) {
                                    app.apply();
                                } else {
                                    app.input_newline();
                                }
                            }
                            KeyCode::Char(ch) => app.input_insert(ch),
                            KeyCode::Backspace => app.input_backspace(),
                            KeyCode::Delete => app.input_delete(),
                            KeyCode::Left => app.input_move_left(),
                            KeyCode::Right => app.input_move_right(),
                            KeyCode::Up => app.input_move_up(),
                            KeyCode::Down => app.input_move_down(),
                            KeyCode::Home => app.input_home(),
                            KeyCode::End => app.input_end(),
                            _ => {}
                        },

                        Focus::AppList => match key.code {
                            KeyCode::Char('q') => return Ok(()),
                            KeyCode::Up | KeyCode::Char('k') => app.move_list(-1),
                            KeyCode::Down | KeyCode::Char('j') => app.move_list(1),
                            KeyCode::Char(' ') | KeyCode::Enter => app.toggle_selected(),
                            KeyCode::Tab => app.focus = Focus::Buttons,
                            KeyCode::PageUp => app.preview_scroll = app.preview_scroll.saturating_sub(5),
                            KeyCode::PageDown => app.preview_scroll += 5,
                            _ => {}
                        },

                        Focus::Buttons => match key.code {
                            KeyCode::Char('q') => return Ok(()),
                            KeyCode::Left | KeyCode::Char('h') => app.button_focus = ButtonFocus::Apply,
                            KeyCode::Right | KeyCode::Char('l') => app.button_focus = ButtonFocus::Quit,
                            KeyCode::Enter | KeyCode::Char(' ') => match app.button_focus {
                                ButtonFocus::Apply => app.apply(),
                                ButtonFocus::Quit => return Ok(()),
                            },
                            KeyCode::Tab => app.focus = Focus::Input,
                            KeyCode::Esc => app.focus = Focus::AppList,
                            _ => {}
                        },
                    }
                }

                Event::Mouse(me) => {
                    if app.handle_mouse(me.column, me.row, me.kind) {
                        drain_events();
                        return Ok(());
                    }
                }

                _ => {}
            }
        }
    }
}


