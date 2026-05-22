use chrono::Local;
use crossterm::{
    event::{self, Event, KeyCode, KeyEventKind, KeyModifiers},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{
    backend::CrosstermBackend,
    layout::{Alignment, Constraint, Direction, Layout},
    style::{Color, Style},
    text::{Line, Span, Text},
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
    AddingEntry,
    EditingEntry(usize),
    ViewingEntry(usize),
}

pub struct App {
    pub entries: Vec<Entry>,
    pub list_state: ListState,
    mode: AppMode,
    new_name: String,
    new_description: TextArea<'static>,
    new_mood: String,
    view_offset: u16,
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
            view_offset: 0,
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
        self.view_offset = 0;
    }

    fn populate_form_from_entry(&mut self, idx: usize) {
        let entry = &self.entries[idx];
        self.new_name = entry.name.clone();
        let mut textarea = TextArea::default();
        textarea.set_placeholder_text("Write your journal entry here...\n(use arrows, backspace, etc.)");
        textarea.set_wrap_mode(WrapMode::WordOrGlyph);
        textarea.clear();
        textarea.insert_str(&entry.description);
        self.new_description = textarea;
        self.new_mood = entry.mood.clone();
        self.view_offset = 0;
    }

    pub fn start_edit(&mut self) {
        if let Some(idx) = self.list_state.selected() {
            self.populate_form_from_entry(idx);
            self.mode = AppMode::EditingEntry(idx);
        }
    }

    pub fn start_view(&mut self) {
        if let Some(idx) = self.list_state.selected() {
            self.view_offset = 0;
            self.mode = AppMode::ViewingEntry(idx);
        }
    }

    pub fn scroll_view(&mut self, delta: i16) {
        let selected = match self.mode {
            AppMode::ViewingEntry(idx) => idx,
            _ => return,
        };
        let entry = &self.entries[selected];
        let height = entry.description.lines().count() as i16;
        if delta < 0 {
            let offset = self.view_offset as i16 + delta;
            self.view_offset = offset.max(0) as u16;
        } else {
            let max_offset = height.saturating_sub(1);
            let offset = self.view_offset as i16 + delta;
            self.view_offset = offset.min(max_offset) as u16;
        }
    }

    fn save_current_entry(&mut self, path: &str) {
        let entry = Entry {
            name: self.new_name.clone(),
            description: self.new_description.lines().join("\n"),
            mood: self.new_mood.clone(),
            time: Local::now().to_string(),
        };

        match self.mode {
            AppMode::AddingEntry => {
                self.entries.push(entry);
                self.list_state.select(Some(self.entries.len() - 1));
            }
            AppMode::EditingEntry(idx) => {
                self.entries[idx] = entry;
                self.list_state.select(Some(idx));
            }
            _ => unreachable!(),
        }
        self.save(path);
        self.mode = AppMode::Normal;
        self.reset_entry_form();
    }

    fn save(&self, path: &str) {
        let updated = serde_json::to_string_pretty(&self.entries).unwrap();
        fs::write(path, updated).unwrap();
    }

    pub fn remove_entry(&mut self) {
        if let Some(index) = self.list_state.selected() {
            self.entries.remove(index);
            if !self.entries.is_empty() {
                let new_index = if index == 0 { 0 } else { index - 1 };
                self.list_state.select(Some(new_index));
            } else {
                self.list_state.select(None);
            }
            self.save("journal.json");
        }
    }

    fn reading_entry(&self) -> Option<&Entry> {
        if let Some(idx) = self.list_state.selected() {
            self.entries.get(idx)
        } else {
            None
        }
    }
}

fn draw_main_ui(f: &mut Frame, app: &mut App) {
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
        Span::styled("Enter/v", Style::default().fg(Color::Yellow).bold()),
        Span::raw(" view  | "),
        Span::styled("q", Style::default().fg(Color::Yellow).bold()),
        Span::raw(" quit  | "),
        Span::styled("a", Style::default().fg(Color::Yellow).bold()),
        Span::raw(" add entry | "),
        Span::styled("e", Style::default().fg(Color::Yellow).bold()),
        Span::raw(" edit entry | "),
        Span::styled("r/delete", Style::default().fg(Color::Yellow).bold()),
        Span::raw(" remove entry"),
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
    f.render_stateful_widget(list, chunks[2], &mut app.list_state);
}

fn draw_entry_screen(f: &mut Frame, app: &mut App) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .margin(2)
        .constraints([
            Constraint::Length(1),
            Constraint::Length(3),
            Constraint::Min(8),
            Constraint::Length(3),
            Constraint::Length(1),
        ])
        .split(f.area());

    let title = match app.mode {
        AppMode::AddingEntry => "=== Adding new entry ===",
        AppMode::EditingEntry(_) => "=== Editing entry ===",
        _ => "",
    };
    let instruct = Paragraph::new(title).alignment(Alignment::Center);
    f.render_widget(instruct, chunks[0]);

    let name_text = format!("Name: {}", app.new_name);
    let name_para = Paragraph::new(name_text).block(Block::default().borders(Borders::NONE));
    f.render_widget(name_para, chunks[1]);

    f.render_widget(&app.new_description, chunks[2]);

    let mood_text = format!("Mood: {}", app.new_mood);
    let mood_para = Paragraph::new(mood_text).block(Block::default().borders(Borders::NONE));
    f.render_widget(mood_para, chunks[3]);

    let help = Paragraph::new("Enter: finish & save | Esc: cancel | Tab: switch fields | Ctrl+J: new line")
        .style(Style::default().fg(Color::DarkGray))
        .alignment(Alignment::Center);
    f.render_widget(help, chunks[4]);
}

fn draw_view_screen(f: &mut Frame, app: &mut App) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .margin(2)
        .constraints([
            Constraint::Length(1),
            Constraint::Length(3),
            Constraint::Min(8),
            Constraint::Length(2),
        ])
        .split(f.area());

    if let Some(entry) = app.reading_entry() {
        let title = format!("=== Viewing entry ===");
        let title_para = Paragraph::new(title).alignment(Alignment::Center);
        f.render_widget(title_para, chunks[0]);

        let metadata = format!("Name: {}  |  Mood: {}  |  Time: {}", entry.name, entry.mood, entry.time);
        let meta_para = Paragraph::new(metadata)
            .block(Block::default().borders(Borders::ALL).title("Entry info"));
        f.render_widget(meta_para, chunks[1]);

        let description = Text::from(entry.description.clone());
        let description_para = Paragraph::new(description)
            .block(Block::default().borders(Borders::ALL).title("Description"))
            .scroll((app.view_offset, 0));
        f.render_widget(description_para, chunks[2]);

        let help = Paragraph::new("↑/↓ scroll  |  Esc back  |  q quit")
            .style(Style::default().fg(Color::DarkGray))
            .alignment(Alignment::Center);
        f.render_widget(help, chunks[3]);
    }
}

pub fn tui(f: &mut Frame, app: &mut App) {
    match app.mode {
        AppMode::Normal => draw_main_ui(f, app),
        AppMode::AddingEntry | AppMode::EditingEntry(_) => draw_entry_screen(f, app),
        AppMode::ViewingEntry(_) => draw_view_screen(f, app),
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

    #[derive(PartialEq)]
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
                            KeyCode::Char('e') => {
                                app.start_edit();
                                focus = Field::Name;
                            }
                            KeyCode::Char('v') | KeyCode::Enter => {
                                app.start_view();
                            }
                            KeyCode::Down => app.next(),
                            KeyCode::Up => app.previous(),
                            KeyCode::Delete => app.remove_entry(),
                            KeyCode::Char('r') => app.remove_entry(),
                            _ => {}
                        },
                        AppMode::ViewingEntry(_) => match key.code {
                            KeyCode::Esc => {
                                app.mode = AppMode::Normal;
                            }
                            KeyCode::Char('q') => break,
                            KeyCode::Down => app.scroll_view(1),
                            KeyCode::Up => app.scroll_view(-1),
                            _ => {}
                        },
                        AppMode::AddingEntry | AppMode::EditingEntry(_) => match key.code {
                            KeyCode::Esc => {
                                app.mode = AppMode::Normal;
                            }
                            KeyCode::Char('j') if key.modifiers.contains(KeyModifiers::CONTROL) && focus == Field::Description => {
                                app.new_description.insert_str("\n");
                            }
                            KeyCode::Enter => {
                                if !app.new_name.is_empty()
                                    && !app.new_description.lines().join("\n").is_empty()
                                    && !app.new_mood.is_empty()
                                {
                                    app.save_current_entry(path);
                                    if let AppMode::EditingEntry(_) = app.mode {
                                        break;
                                    }
                                }
                            }
                            KeyCode::Tab => {
                                focus = match focus {
                                    Field::Name => Field::Description,
                                    Field::Description => Field::Mood,
                                    Field::Mood => Field::Name,
                                };
                            }
                            _ => {
                                match focus {
                                    Field::Name => match key.code {
                                        KeyCode::Char(c) => app.new_name.push(c),
                                        KeyCode::Backspace => { app.new_name.pop(); }
                                        _ => {}
                                    },
                                    Field::Description => {
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

pub fn run_edit_entry(entry_index: usize, path: &str) {
    let entries: Vec<Entry> = fs::read_to_string(path)
        .ok()
        .and_then(|data| serde_json::from_str(&data).ok())
        .unwrap_or_default();

    let mut app = App::new(entries);
    if entry_index < app.entries.len() {
        app.list_state.select(Some(entry_index));
        app.start_edit();
    } else {
        eprintln!("Index out of range");
        return;
    }

    enable_raw_mode().unwrap();
    let mut stdout = stdout();
    execute!(stdout, EnterAlternateScreen).unwrap();
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend).unwrap();

    #[derive(PartialEq)]
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
                            KeyCode::Char('e') => {
                                app.start_edit();
                                focus = Field::Name;
                            }
                            KeyCode::Char('v') | KeyCode::Enter => {
                                app.start_view();
                            }
                            KeyCode::Down => app.next(),
                            KeyCode::Up => app.previous(),
                            KeyCode::Delete => app.remove_entry(),
                            KeyCode::Char('r') => app.remove_entry(),
                            _ => {}
                        },
                        AppMode::ViewingEntry(_) => match key.code {
                            KeyCode::Esc => {
                                app.mode = AppMode::Normal;
                            }
                            KeyCode::Char('q') => break,
                            KeyCode::Down => app.scroll_view(1),
                            KeyCode::Up => app.scroll_view(-1),
                            _ => {}
                        },
                        AppMode::AddingEntry | AppMode::EditingEntry(_) => match key.code {
                            KeyCode::Esc => {
                                app.mode = AppMode::Normal;
                            }
                            KeyCode::Char('j') if key.modifiers.contains(KeyModifiers::CONTROL) && focus == Field::Description => {
                                app.new_description.insert_str("\n");
                            }
                            KeyCode::Enter => {
                                if !app.new_name.is_empty()
                                    && !app.new_description.lines().join("\n").is_empty()
                                    && !app.new_mood.is_empty()
                                {
                                    app.save_current_entry(path);
                                    break;
                                }
                            }
                            KeyCode::Tab => {
                                focus = match focus {
                                    Field::Name => Field::Description,
                                    Field::Description => Field::Mood,
                                    Field::Mood => Field::Name,
                                };
                            }
                            _ => {
                                match focus {
                                    Field::Name => match key.code {
                                        KeyCode::Char(c) => app.new_name.push(c),
                                        KeyCode::Backspace => { app.new_name.pop(); }
                                        _ => {}
                                    },
                                    Field::Description => {
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