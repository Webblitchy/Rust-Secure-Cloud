use crossterm::event::{self, Event, KeyCode, KeyEventKind};
use ratatui::prelude::{Alignment, Rect};
use ratatui::style::{Color, Style};
use ratatui::text::Text;
use ratatui::Terminal;
use ratatui::{backend::CrosstermBackend, style::Modifier};
use ratatui::{
    layout::{Constraint, Layout},
    prelude::Direction,
};
use ratatui::{widgets::*, Frame};
use std::io;
use std::io::StdoutLock;
use tui_textarea::{Input, Key, TextArea};

use crate::inputs::validate_input;
use crate::structs::ValidationType;

#[derive(PartialEq)] // enable comparison with ==
pub enum PopupType {
    Disabled,
    Info,
    Error,
}

pub struct Interface<'a> {
    pub term: Terminal<CrosstermBackend<StdoutLock<'a>>>,
    popup_val: Text<'a>,
    popup_type: PopupType,
}

impl Interface<'_> {
    pub fn new(term: Terminal<CrosstermBackend<StdoutLock<'_>>>) -> Interface<'_> {
        Interface {
            term,
            popup_val: Text::from(""),
            popup_type: PopupType::Disabled,
        }
    }

    pub fn set_popup(&mut self, text: &str, popup_type: PopupType) {
        self.popup_val = Text::from(text.to_string());
        self.popup_type = popup_type;
    }

    pub fn show_popup(&mut self) {
        let border_style;
        let title;
        match self.popup_type {
            PopupType::Info => {
                border_style = Style::default().fg(Color::White);
                title = "Info";
            }
            PopupType::Error => {
                border_style = Style::default().fg(Color::LightRed);
                title = "Error";
            }
            PopupType::Disabled => return,
        }
        self.term
            .draw(|f| {
                let paragraph = Paragraph::new(self.popup_val.clone())
                    .wrap(Wrap { trim: true })
                    .alignment(Alignment::Center);
                let block = Block::default()
                    .title(title)
                    .borders(Borders::ALL)
                    .border_style(border_style)
                    .border_type(BorderType::Rounded);
                let area = centered_rect(50, 30, f.size());
                f.render_widget(Clear, area); //this clears out the background
                f.render_widget(paragraph.clone().block(block), area);
            })
            .unwrap();
    }

    pub fn hide_popup(&mut self) {
        self.popup_type = PopupType::Disabled;
    }
}

pub fn input_field(
    interface: &mut Interface<'_>,
    title: &str,
    validation_type: &ValidationType,
) -> io::Result<String> {
    let mut textarea = TextArea::default();
    textarea.set_cursor_line_style(Style::default());
    textarea.set_block(
        Block::default()
            .borders(Borders::ALL)
            .border_type(BorderType::Rounded)
            .style(Style::default())
            .title(title),
    );
    let layout =
        Layout::default().constraints([Constraint::Length(3), Constraint::Min(1)].as_slice());
    let mut is_valid = validate_input(&mut textarea, validation_type);

    loop {
        if interface.popup_type != PopupType::Disabled {
            interface.show_popup(); // shown only if enabled
            if let Event::Key(key) = event::read().unwrap() {
                if key.kind == KeyEventKind::Press {
                    interface.hide_popup();
                }
            }
        } else {
            interface.term.draw(|f: &mut Frame<'_>| {
                let chunks = layout.split(f.size());
                let widget = textarea.widget();
                f.render_widget(Clear, chunks[0]);
                f.render_widget(widget, chunks[0]);
            })?;

            match crossterm::event::read()?.into() {
                Input { key: Key::Esc, .. } => break,
                Input {
                    key: Key::Enter, ..
                } if is_valid => break,
                Input {
                    key: Key::Char('m'),
                    ctrl: true,
                    ..
                }
                | Input {
                    key: Key::Enter, ..
                } => {}
                input => {
                    // TextArea::input returns if the input modified its text
                    if textarea.input(input) {
                        is_valid = validate_input(&mut textarea, validation_type);
                    }
                }
            }
        }
    }

    interface.term.show_cursor()?;

    let entry = textarea.lines()[0].clone();
    Ok(entry)
}

fn inactivate_input(textarea: &mut TextArea<'_>) {
    textarea.set_cursor_line_style(Style::default());
    textarea.set_cursor_style(Style::default());

    // to keep title
    let old_block = textarea.block().unwrap().clone();
    textarea.set_block(
        old_block
            .borders(Borders::ALL)
            .border_type(BorderType::Rounded)
            .style(Style::default().fg(Color::DarkGray)),
    );
}

fn activate_input(textarea: &mut TextArea<'_>) {
    textarea.set_cursor_line_style(Style::default().add_modifier(Modifier::UNDERLINED));
    textarea.set_cursor_style(Style::default().add_modifier(Modifier::REVERSED));

    // to keep title
    let old_block = textarea.block().unwrap().clone();
    textarea.set_block(
        old_block
            .borders(Borders::ALL)
            .border_type(BorderType::Rounded)
            .style(Style::default()),
    );
}

pub fn user_passwd_input(
    interface: &mut Interface<'_>,
    user_nb: usize,
    validate_password: bool,
) -> io::Result<(String, String)> {
    let mut textareas = [TextArea::default(), TextArea::default()];
    let layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints(
            [
                Constraint::Length(3),
                Constraint::Length(3),
                Constraint::Min(0),
            ]
            .as_ref(),
        );

    textareas[0].set_block(
        Block::default()
            .borders(Borders::ALL)
            .border_type(BorderType::Rounded)
            .title(format!("Username {}", user_nb)),
    );
    textareas[1].set_block(
        Block::default()
            .borders(Borders::ALL)
            .border_type(BorderType::Rounded)
            .title(format!("Password {}", user_nb)),
    );
    textareas[1].set_mask_char('\u{2022}'); // U+2022 BULLET (•)
    activate_input(&mut textareas[0]);
    inactivate_input(&mut textareas[1]);

    let mut input_no = 0;
    let mut is_valid = validate_input(&mut textareas[0], &ValidationType::NotEmpty);

    loop {
        if interface.popup_type != PopupType::Disabled {
            interface.show_popup(); // shown only if enabled
            if let Event::Key(key) = event::read()? {
                if key.kind == KeyEventKind::Press {
                    interface.hide_popup();
                }
            }
        } else {
            interface.term.draw(|f| {
                let chunks = layout.split(f.size());
                for (textarea, chunk) in textareas.iter().zip(chunks.iter()) {
                    let widget = textarea.widget();
                    f.render_widget(Clear, *chunk);
                    f.render_widget(widget, *chunk);
                }
            })?;

            match crossterm::event::read()?.into() {
                Input { key: Key::Esc, .. } => break,
                Input {
                    key: Key::Enter, ..
                } if is_valid => {
                    if input_no == 0 {
                        inactivate_input(&mut textareas[input_no]);
                        input_no = (input_no + 1) % 2;
                        activate_input(&mut textareas[input_no]);
                    } else {
                        break;
                    }
                }
                Input { key: Key::Tab, .. }
                | Input { key: Key::Up, .. }
                | Input { key: Key::Down, .. } => {
                    if is_valid {
                        inactivate_input(&mut textareas[input_no]);
                        input_no = (input_no + 1) % 2;
                        activate_input(&mut textareas[input_no]);
                    }
                }
                Input {
                    // ignore Enter if not valid
                    key: Key::Enter,
                    ..
                } => {}
                input => {
                    // TextArea::input returns if the input modified its text
                    if textareas[input_no].input(input) {
                        let validation_type = match input_no {
                            1 if validate_password => ValidationType::Password,
                            _ => ValidationType::NotEmpty,
                        };
                        is_valid = validate_input(&mut textareas[input_no], &validation_type);
                    }
                }
            }
        }
    }

    interface.term.show_cursor()?;

    let username = textareas[0].lines()[0].clone();
    let password = textareas[1].lines()[0].clone();
    Ok((username, password))
}

struct StatefulList<T> {
    state: ListState,
    items: Vec<T>,
}

impl<T> StatefulList<T> {
    fn new(items: Vec<T>) -> StatefulList<T> {
        StatefulList {
            state: ListState::default().with_selected(Some(0)),
            items,
        }
    }

    fn next(&mut self) {
        let i = match self.state.selected() {
            Some(i) => {
                if i >= self.items.len() - 1 {
                    0
                } else {
                    i + 1
                }
            }
            None => 0,
        };
        self.state.select(Some(i));
    }

    fn previous(&mut self) {
        let i = match self.state.selected() {
            Some(i) => {
                if i == 0 {
                    self.items.len() - 1
                } else {
                    i - 1
                }
            }
            None => 0,
        };
        self.state.select(Some(i));
    }
}

pub fn choice_list(
    interface: &mut Interface<'_>,
    choices: Vec<String>,
) -> io::Result<Option<usize>> {
    let shown_items: Vec<ListItem> = choices
        .iter()
        .map(|item| ListItem::new(item.clone()).style(Style::default().fg(Color::Blue)))
        .collect();

    // Create a List from all list items and highlight the currently selected one
    let shown_items = List::new(shown_items)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_type(BorderType::Rounded)
                .title("Secure Cloud"),
        )
        .highlight_style(
            Style::default()
                .fg(Color::LightBlue)
                .bg(Color::DarkGray)
                .add_modifier(Modifier::BOLD),
        )
        .highlight_symbol(" > ");

    let mut choice_list = StatefulList::new(choices);

    loop {
        if interface.popup_type != PopupType::Disabled {
            interface.show_popup();
            if let Event::Key(key) = event::read()? {
                if key.kind == KeyEventKind::Press {
                    interface.hide_popup();
                }
            }
        } else {
            interface.term.draw(|f| {
                let chunks = Layout::default()
                    .direction(Direction::Horizontal)
                    .constraints([Constraint::Percentage(100)])
                    .split(f.size());

                f.render_widget(Clear, chunks[0]);
                f.render_stateful_widget(shown_items.clone(), chunks[0], &mut choice_list.state);
            })?;

            if let Event::Key(key) = event::read()? {
                if key.kind == KeyEventKind::Press {
                    match key.code {
                        KeyCode::Esc | KeyCode::Char('q') => return Ok(Option::None),
                        KeyCode::Down | KeyCode::Char('j') | KeyCode::Tab => choice_list.next(),
                        KeyCode::Up | KeyCode::Char('k') | KeyCode::BackTab => {
                            choice_list.previous()
                        }
                        KeyCode::Enter => return Ok(choice_list.state.selected()),
                        _ => {}
                    }
                }
            }
        }
    }
}

/// helper function to create a centered rect using up certain percentage of the available rect `r`
fn centered_rect(percent_x: u16, percent_y: u16, r: Rect) -> Rect {
    let popup_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage((100 - percent_y) / 2),
            Constraint::Percentage(percent_y),
            Constraint::Percentage((100 - percent_y) / 2),
        ])
        .split(r);

    Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage((100 - percent_x) / 2),
            Constraint::Percentage(percent_x),
            Constraint::Percentage((100 - percent_x) / 2),
        ])
        .split(popup_layout[1])[1]
}
