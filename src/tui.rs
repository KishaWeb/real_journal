use crate::quick_journal::Entry;
use std::fs;
use std::time::Duration;
use crossterm::{
    execute,
    event::{self, Event, KeyCode},
    terminal::{enable_raw_mode, disable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use std::io::stdout;
use serde_json;
use ratatui::backend::CrosstermBackend;
use ratatui::{
    widgets::{Block, Borders, List, ListItem, ListState, Paragraph},
    layout::{Constraint, Layout, Direction},
    style::{Style, Color},
    text::{Line, Span},
    Frame, Terminal,
};

pub struct App {
    pub entries: Vec<Entry>,
    pub list_state: ListState,
}

impl App {
    pub fn new(entries: Vec<Entry>) -> Self {
        let mut list_state = ListState::default();
        if !entries.is_empty() {
            list_state.select(Some(0));
        }
        App {
            entries,
            list_state,
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
}

pub fn tui(f: &mut Frame, app: &mut App) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .margin(2)
        .constraints([
            Constraint::Length(1),
            Constraint::Length(3),
            Constraint::Min(2),
        ])
        .split(f.size());

    let help_text = Line::from(vec![
        Span::styled("↑/↓", Style::default().fg(Color::Yellow).bold()),
        Span::raw(" navigate  | "),
        Span::styled("q", Style::default().fg(Color::Yellow).bold()),
        Span::raw(" quit"),
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

    loop {
        terminal.draw(|f| tui(f, &mut app)).unwrap();

        if event::poll(Duration::from_millis(200)).unwrap() {
            if let Event::Key(key) = event::read().unwrap() {
                match key.code {
                    KeyCode::Char('q') => break,
                    KeyCode::Down => app.next(),
                    KeyCode::Up => app.previous(),
                    _ => {}
                }
            }
        }
    }

    disable_raw_mode().unwrap();
    execute!(terminal.backend_mut(), LeaveAlternateScreen).unwrap();
    terminal.show_cursor().unwrap();
}