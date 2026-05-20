use chrono::Local;
use crossterm::{
    event::{self, Event, KeyCode, KeyEventKind},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{
    backend::CrosstermBackend,
    layout::{Alignment, Constraint, Direction, Layout},
    style::{Color, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, ListState, Paragraph},
    Frame, Terminal,
};
use ratatui_textarea::{TextArea, Input, WrapMode};
use serde::{Deserialize, Serialize};
use std::fs;
use std::io::stdout;
use std::time::Duration;

#[derive(Serialize, Deserialize, Clone)]
pub struct Entry {
    pub name: String,
    pub description: String,
    pub mood: String,
    pub time: String,
}

#[derive(PartialEq)]
enum AppMode {
    Normal,
    AddingEntry,   // entry creation screen
}

pub struct App {
    pub entries: Vec<Entry>,
    pub list_state: ListState,
    mode: AppMode,
    // Fields for the new entry
    new_name: String,
    new_description: TextArea<'static>,  // multi‑line editor
    new_mood: String,
}

impl App {
    pub fn new(entries: Vec<Entry>) -> Self {
        let mut list_state = ListState::default();
        if !entries.is_empty() {
            list_state.select(Some(0));
        }
        let mut textarea = TextArea::default();
        textarea.set_cursor_line_style(Style::default());
        textarea.set_placeholder_text("Write your journal entry here...\n(use arrows, backspace, etc.)");
        textarea.set_wrap_mode(WrapMode::WordOrGlyph);
        App {
            entries,
            list_state,
            mode: AppMode::Normal,
            new_name: String::new(),
            new_description: textarea,
            new_mood: String::new(),
        }
    }

    pub fn next(&mut self) {
        if self.entries.is_empty() {
            return;
        }
        let i = self.list_state.selected().unwrap_or(0);
        let next = (i + 1) % self.entries.len();
        self.list_state.select(Some(next));
    }

    pub fn previous(&mut self) {
        if self.entries.is_empty() {
            return;
        }
        let i = self.list_state.selected().unwrap_or(0);
        let prev = if i == 0 { self.entries.len() - 1 } else { i - 1 };
        self.list_state.select(Some(prev));
    }

    fn reset_entry_form(&mut self) {
        self.new_name.clear();
        self.new_description = TextArea::default();
        self.new_description.set_placeholder_text("Write your journal entry here...\n(use arrows, backspace, etc.)");
        self.new_description.set_wrap_mode(WrapMode::WordOrGlyph);
        self.new_mood.clear();
    }

    fn save_entry(&mut self, path: &str) {
        let entry = Entry {
            name: self.new_name.clone(),
            description: self.new_description.lines().join("\n"),
            mood: self.new_mood.clone(),
            time: Local::now().to_string(),
        };
        self.entries.push(entry);
        self.list_state.select(Some(self.entries.len() - 1));
        self.save(path);
    }

    fn save(&self, path: &str) {
        let updated = serde_json::to_string_pretty(&self.entries).unwrap();
        fs::write(path, updated).unwrap();
    }
}

fn draw_main_ui(f: &mut Frame, app: &App) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .margin(2)
        .constraints([
            Constraint::Length(1),
            Constraint::Length(3),
            Constraint::Min(2),
        ])
        .split(f.area());

    let help_text = Line::from(vec![
        Span::styled("↑/↓", Style::default().fg(Color::Yellow).bold()),
        Span::raw(" navigate  | "),
        Span::styled("q", Style::default().fg(Color::Yellow).bold()),
        Span::raw(" quit  | "),
        Span::styled("a", Style::default().fg(Color::Yellow).bold()),
        Span::raw(" add entry"),
    ]);
    let help_paragraph = Paragraph::new(help_text).block(Block::default());
    f.render_widget(help_paragraph, chunks[0]);

    let title_block = Block::default()
        .title("Journal CLI")
        .borders(Borders::ALL);
    f.render_widget(title_block, chunks[1]);

    let items: Vec<ListItem> = app
        .entries
        .iter()
        .enumerate()
        .map(|(i, entry)| {
            let content = format!("{}) {} | {}", i + 1, entry.name, entry.mood);
            ListItem::new(content)
        })
        .collect();

    let list = List::new(items)
        .block(Block::default().title("Entries").borders(Borders::ALL))
        .highlight_symbol(">> ");
    f.render_stateful_widget(list, chunks[2], &mut app.list_state.clone());
}

fn draw_entry_screen(f: &mut Frame, app: &mut App) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .margin(2)
        .constraints([
            Constraint::Length(1),   // instructions
            Constraint::Length(3),   // name field
            Constraint::Min(8),      // description (textarea)
            Constraint::Length(3),   // mood field
            Constraint::Length(1),   // help line
        ])
        .split(f.area());

    // Instructions
    let instruct = Paragraph::new("=== Adding new entry ===").alignment(Alignment::Center);
    f.render_widget(instruct, chunks[0]);

    // Name field
    let name_text = format!("Name: {}", app.new_name);
    let name_para = Paragraph::new(name_text).block(Block::default().borders(Borders::NONE));
    f.render_widget(name_para, chunks[1]);

    // Description (TextArea) – handles wrapping + scrolling automatically
    f.render_widget(&app.new_description, chunks[2]);

    // Mood field
    let mood_text = format!("Mood: {}", app.new_mood);
    let mood_para = Paragraph::new(mood_text).block(Block::default().borders(Borders::NONE));
    f.render_widget(mood_para, chunks[3]);

    // Help line
    let help = Paragraph::new("Enter: finish & save | Esc: cancel | Tab: switch fields")
        .style(Style::default().fg(Color::DarkGray))
        .alignment(Alignment::Center);
    f.render_widget(help, chunks[4]);
}

pub fn tui(f: &mut Frame, app: &mut App) {
    match app.mode {
        AppMode::Normal => draw_main_ui(f, app),
        AppMode::AddingEntry => draw_entry_screen(f, app),
    }
}

pub fn run() {
    let path = "journal.json";
    let entries: Vec<Entry> = fs::read_to_string(path)
        .ok()
        .and_then(|data| serde_json::from_str(&data).ok())
        .unwrap_or_default();

    let mut app = App::new(entries);

    enable_raw_mode().unwrap();
    let mut stdout = stdout();
    execute!(stdout, EnterAlternateScreen).unwrap();
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend).unwrap();

    // We'll manage focus manually (simple)
    enum Field { Name, Description, Mood }
    let mut focus = Field::Name;

    loop {
        terminal.draw(|f| tui(f, &mut app)).unwrap();

        if event::poll(Duration::from_millis(100)).unwrap() {
            if let Event::Key(key) = event::read().unwrap() {
                if key.kind == KeyEventKind::Press {
                    match app.mode {
                        AppMode::Normal => match key.code {
                            KeyCode::Char('q') => break,
                            KeyCode::Char('a') => {
                                app.reset_entry_form();
                                app.mode = AppMode::AddingEntry;
                                focus = Field::Name;
                            }
                            KeyCode::Down => app.next(),
                            KeyCode::Up => app.previous(),
                            _ => {}
                        },
                        AppMode::AddingEntry => match key.code {
                            KeyCode::Esc => {
                                app.mode = AppMode::Normal;
                            }
                            KeyCode::Enter => {
                                // Save if all fields non‑empty
                                if !app.new_name.is_empty()
                                    && !app.new_description.lines().join("\n").is_empty()
                                    && !app.new_mood.is_empty()
                                {
                                    app.save_entry(path);
                                    app.mode = AppMode::Normal;
                                }
                            }
                            KeyCode::Tab => {
                                // Cycle focus
                                focus = match focus {
                                    Field::Name => Field::Description,
                                    Field::Description => Field::Mood,
                                    Field::Mood => Field::Name,
                                };
                            }
                            _ => {
                                // Handle input based on current focus
                                match focus {
                                    Field::Name => match key.code {
                                        KeyCode::Char(c) => app.new_name.push(c),
                                        KeyCode::Backspace => { app.new_name.pop(); }
                                        _ => {}
                                    },
                                    Field::Description => {
                                        // Let TextArea handle the event
                                        app.new_description.input(Input::from(key));
                                    }
                                    Field::Mood => match key.code {
                                        KeyCode::Char(c) => app.new_mood.push(c),
                                        KeyCode::Backspace => { app.new_mood.pop(); }
                                        _ => {}
                                    },
                                }
                            }
                        },
                    }
                }
            }
        }
    }

    disable_raw_mode().unwrap();
    execute!(terminal.backend_mut(), LeaveAlternateScreen).unwrap();
    terminal.show_cursor().unwrap();
}