use std::collections::HashMap;
use std::time::{Duration, Instant};

use crossterm::event::{MouseButton, MouseEventKind};
use ratatui::layout::Rect;
use ratatui::widgets::ListState;

use crate::vmoptions::{append_vmoptions, find_apps, resolve_options_path};

#[derive(Debug, Clone, PartialEq)]
pub(crate) enum Focus {
    AppList,
    Buttons,
    Input,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub(crate) enum ButtonFocus {
    Apply,
    Quit,
}

pub(crate) struct Notification {
    pub(crate) message: String,
    pub(crate) severity: Severity,
    pub(crate) born: Instant,
}

#[derive(Debug, Clone, Copy)]
pub(crate) enum Severity {
    Info,
    Warning,
    Error,
}

#[derive(Default, Clone)]
pub(crate) struct AppLayout {
    pub(crate) list_area: Rect,
    pub(crate) apply_area: Rect,
    pub(crate) quit_area: Rect,
    pub(crate) preview_area: Rect,
    pub(crate) input_area: Rect,
}

pub(crate) struct App {
    pub(crate) app_names: Vec<String>,
    pub(crate) apps: HashMap<String, String>,
    pub(crate) check_states: Vec<bool>,
    pub(crate) list_state: ListState,
    pub(crate) focus: Focus,
    pub(crate) button_focus: ButtonFocus,
    pub(crate) notifications: Vec<Notification>,
    pub(crate) preview_scroll: u16,
    pub(crate) layout: AppLayout,
    pub(crate) lines: Vec<String>,
    pub(crate) cursor_row: usize,
    pub(crate) cursor_col: usize,
    pub(crate) input_scroll: usize,
}

impl App {
    pub(crate) fn new(initial: Option<String>) -> Self {
        let apps = find_apps();
        let mut app_names: Vec<String> = apps.keys().cloned().collect();
        app_names.sort();
        let check_count = app_names.len() + 1;
        let mut list_state = ListState::default();
        list_state.select(Some(0));

        let lines: Vec<String> = match initial {
            Some(s) => s.lines().map(|l| l.to_string()).collect(),
            None => vec![String::new()],
        };
        let lines = if lines.is_empty() { vec![String::new()] } else { lines };
        let cursor_row = lines.len() - 1;
        let cursor_col = lines[cursor_row].len();

        Self {
            app_names,
            apps,
            check_states: vec![false; check_count],
            list_state,
            focus: Focus::AppList,
            button_focus: ButtonFocus::Apply,
            notifications: Vec::new(),
            preview_scroll: 0,
            layout: AppLayout::default(),
            lines,
            cursor_row,
            cursor_col,
            input_scroll: 0,
        }
    }

    pub(crate) fn options(&self) -> Vec<String> {
        self.lines
            .iter()
            .map(|l| l.trim().to_string())
            .filter(|l| !l.is_empty())
            .collect()
    }

    pub(crate) fn ensure_cursor_visible(&mut self, inner_width: usize) {
        if inner_width == 0 {
            return;
        }
        let col = self.cursor_col;
        if col < self.input_scroll {
            self.input_scroll = col;
        } else if col >= self.input_scroll + inner_width {
            self.input_scroll = col - inner_width + 1;
        }
    }

    pub(crate) fn input_insert(&mut self, ch: char) {
        self.lines[self.cursor_row].insert(self.cursor_col, ch);
        self.cursor_col += ch.len_utf8();
    }

    pub(crate) fn input_newline(&mut self) {
        let rest = self.lines[self.cursor_row][self.cursor_col..].to_string();
        self.lines[self.cursor_row].truncate(self.cursor_col);
        self.cursor_row += 1;
        self.lines.insert(self.cursor_row, rest);
        self.cursor_col = 0;
        self.input_scroll = 0;
    }

    pub(crate) fn input_backspace(&mut self) {
        if self.cursor_col == 0 {
            if self.cursor_row == 0 {
                return;
            }
            let current = self.lines.remove(self.cursor_row);
            self.cursor_row -= 1;
            self.cursor_col = self.lines[self.cursor_row].len();
            self.lines[self.cursor_row].push_str(&current);
        } else {
            let mut new_col = self.cursor_col - 1;
            while !self.lines[self.cursor_row].is_char_boundary(new_col) {
                new_col -= 1;
            }
            self.lines[self.cursor_row].remove(new_col);
            self.cursor_col = new_col;
        }
    }

    pub(crate) fn input_delete(&mut self) {
        let line_len = self.lines[self.cursor_row].len();
        if self.cursor_col >= line_len {
            if self.cursor_row + 1 < self.lines.len() {
                let next = self.lines.remove(self.cursor_row + 1);
                self.lines[self.cursor_row].push_str(&next);
            }
        } else {
            self.lines[self.cursor_row].remove(self.cursor_col);
        }
    }

    pub(crate) fn input_move_left(&mut self) {
        if self.cursor_col == 0 {
            if self.cursor_row > 0 {
                self.cursor_row -= 1;
                self.cursor_col = self.lines[self.cursor_row].len();
            }
        } else {
            let mut p = self.cursor_col - 1;
            while !self.lines[self.cursor_row].is_char_boundary(p) {
                p -= 1;
            }
            self.cursor_col = p;
        }
    }

    pub(crate) fn input_move_right(&mut self) {
        let line_len = self.lines[self.cursor_row].len();
        if self.cursor_col >= line_len {
            if self.cursor_row + 1 < self.lines.len() {
                self.cursor_row += 1;
                self.cursor_col = 0;
                self.input_scroll = 0;
            }
        } else {
            let mut p = self.cursor_col + 1;
            while p < line_len && !self.lines[self.cursor_row].is_char_boundary(p) {
                p += 1;
            }
            self.cursor_col = p;
        }
    }

    pub(crate) fn input_move_up(&mut self) {
        if self.cursor_row == 0 {
            return;
        }
        self.cursor_row -= 1;
        self.cursor_col = self.cursor_col.min(self.lines[self.cursor_row].len());
        while self.cursor_col > 0
            && !self.lines[self.cursor_row].is_char_boundary(self.cursor_col)
        {
            self.cursor_col -= 1;
        }
    }

    pub(crate) fn input_move_down(&mut self) {
        if self.cursor_row + 1 >= self.lines.len() {
            return;
        }
        self.cursor_row += 1;
        self.cursor_col = self.cursor_col.min(self.lines[self.cursor_row].len());
        while self.cursor_col > 0
            && !self.lines[self.cursor_row].is_char_boundary(self.cursor_col)
        {
            self.cursor_col -= 1;
        }
    }

    pub(crate) fn input_home(&mut self) {
        self.cursor_col = 0;
        self.input_scroll = 0;
    }

    pub(crate) fn input_end(&mut self) {
        self.cursor_col = self.lines[self.cursor_row].len();
    }

    pub(crate) fn apply(&mut self) {
        let options = self.options();

        if options.is_empty() {
            self.notify("No options entered", Severity::Warning);
            return;
        }

        let selected: Vec<(String, String)> = self
            .selected_apps()
            .iter()
            .map(|(n, b)| (n.to_string(), b.to_string()))
            .collect();

        if selected.is_empty() {
            self.notify("No apps selected", Severity::Warning);
            return;
        }

        for (name, binary) in &selected {
            match resolve_options_path(binary) {
                None => self.notify(&format!("{name}: vmoptions not found"), Severity::Error),
                Some(path) => {
                    let mut any_updated = false;
                    for opt in &options {
                        match append_vmoptions(&path, opt) {
                            Ok(true) => {
                                any_updated = true;
                            }
                            Ok(false) => {}
                            Err(e) => {
                                self.notify(&format!("{name}: error – {e}"), Severity::Error);
                            }
                        }
                    }
                    if any_updated {
                        self.notify(&format!("{name}: updated"), Severity::Info);
                    } else {
                        self.notify(&format!("{name}: already set"), Severity::Warning);
                    }
                }
            }
        }
        self.notify("Done", Severity::Info);
    }

    pub(crate) fn notify(&mut self, message: &str, severity: Severity) {
        self.notifications.push(Notification {
            message: message.to_string(),
            severity,
            born: Instant::now(),
        });
        if self.notifications.len() > 5 {
            self.notifications.remove(0);
        }
    }

    pub(crate) fn prune_notifications(&mut self) {
        self.notifications
            .retain(|n| n.born.elapsed() < Duration::from_secs(4));
    }

    pub(crate) fn move_list(&mut self, delta: i32) {
        let len = self.app_names.len() + 1;
        if len == 0 {
            return;
        }
        let current = self.list_state.selected().unwrap_or(0) as i32;
        let next = (current + delta).rem_euclid(len as i32) as usize;
        self.list_state.select(Some(next));
    }

    pub(crate) fn toggle_item(&mut self, i: usize) {
        if i >= self.check_states.len() {
            return;
        }
        let new_val = !self.check_states[i];
        self.check_states[i] = new_val;
        if i == 0 {
            for s in self.check_states.iter_mut() {
                *s = new_val;
            }
        } else if !new_val {
            self.check_states[0] = false;
        } else {
            let all_checked = self.check_states[1..].iter().all(|&v| v);
            self.check_states[0] = all_checked;
        }
    }

    pub(crate) fn toggle_selected(&mut self) {
        if let Some(i) = self.list_state.selected() {
            self.toggle_item(i);
        }
    }

    pub(crate) fn selected_apps(&self) -> Vec<(&str, &str)> {
        if self.check_states[0] {
            return self
                .app_names
                .iter()
                .map(|n| (n.as_str(), self.apps[n].as_str()))
                .collect();
        }
        self.app_names
            .iter()
            .enumerate()
            .filter(|(i, _)| self.check_states[i + 1])
            .map(|(_, n)| (n.as_str(), self.apps[n].as_str()))
            .collect()
    }

    pub(crate) fn preview_text(&self) -> String {
        let selected = self.selected_apps();
        if selected.is_empty() {
            return "No apps selected".to_string();
        }
        selected
            .iter()
            .map(|(name, binary)| {
                let vmopts = resolve_options_path(binary)
                    .map(|p| p.display().to_string())
                    .unwrap_or_else(|| "NOT FOUND".to_string());
                format!("{name}\n  {binary}\n  → {vmopts}\n")
            })
            .collect::<Vec<_>>()
            .join("\n")
    }

    pub(crate) fn list_index_at(&self, col: u16, row: u16) -> Option<usize> {
        let a = self.layout.list_area;
        if col < a.x + 1 || col >= a.x + a.width.saturating_sub(1) {
            return None;
        }
        if row < a.y + 1 || row >= a.y + a.height.saturating_sub(1) {
            return None;
        }
        let inner_row = (row - (a.y + 1)) as usize;
        let total = self.app_names.len() + 1;
        if inner_row < total {
            Some(inner_row)
        } else {
            None
        }
    }

    pub(crate) fn handle_mouse(&mut self, col: u16, row: u16, kind: MouseEventKind) -> bool {
        match kind {
            MouseEventKind::Down(MouseButton::Left) => {
                if let Some(idx) = self.list_index_at(col, row) {
                    self.focus = Focus::AppList;
                    self.list_state.select(Some(idx));
                    self.toggle_item(idx);
                    return false;
                }
                if rect_contains(self.layout.apply_area, col, row) {
                    self.focus = Focus::Buttons;
                    self.button_focus = ButtonFocus::Apply;
                    self.apply();
                    return false;
                }
                if rect_contains(self.layout.quit_area, col, row) {
                    return true;
                }
                if rect_contains(self.layout.input_area, col, row) {
                    self.focus = Focus::Input;
                    let inner_y = row.saturating_sub(self.layout.input_area.y + 1) as usize;
                    let clicked_row = inner_y.min(self.lines.len() - 1);
                    let inner_x = col.saturating_sub(self.layout.input_area.x + 1) as usize;
                    let clicked_col = (inner_x + self.input_scroll).min(self.lines[clicked_row].len());
                    let mut cc = clicked_col;
                    while cc > 0 && !self.lines[clicked_row].is_char_boundary(cc) {
                        cc -= 1;
                    }
                    self.cursor_row = clicked_row;
                    self.cursor_col = cc;
                    return false;
                }
                if self.focus == Focus::Input {
                    self.focus = Focus::AppList;
                }
                false
            }
            MouseEventKind::ScrollUp => {
                if rect_contains(self.layout.list_area, col, row) {
                    self.move_list(-1);
                } else if rect_contains(self.layout.preview_area, col, row) {
                    self.preview_scroll = self.preview_scroll.saturating_sub(1);
                }
                false
            }
            MouseEventKind::ScrollDown => {
                if rect_contains(self.layout.list_area, col, row) {
                    self.move_list(1);
                } else if rect_contains(self.layout.preview_area, col, row) {
                    self.preview_scroll += 1;
                }
                false
            }
            _ => false,
        }
    }
}

fn rect_contains(r: Rect, col: u16, row: u16) -> bool {
    col >= r.x && col < r.x + r.width && row >= r.y && row < r.y + r.height
}



