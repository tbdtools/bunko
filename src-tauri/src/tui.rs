use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode, KeyEventKind},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{
    backend::{Backend, CrosstermBackend},
    layout::{Alignment, Constraint, Direction, Layout},
    style::{Color, Modifier, Style},
    text::{Line, Span, Text},
    widgets::{
        Block, Borders, List, ListItem, ListState, Paragraph, ScrollbarState, Wrap,
    },
    Frame, Terminal,
};
use std::{
    error::Error,
    io,
    path::Path,
    time::{Duration, Instant},
};

use bunko_lib::{notebook, Note};

#[derive(Clone)]
pub enum InputMode {
    Normal,
    Editing,
    Searching,
    Creating,
}

pub struct App {
    pub notebook_path: String,
    pub notes: Vec<Note>,
    pub selected_note_index: Option<usize>,
    pub note_content: String,
    pub input_mode: InputMode,
    pub search_query: String,
    pub create_title: String,
    pub cursor_position: usize,
    pub editor_scroll: usize,
    pub last_save: Option<Instant>,
    pub status_message: String,
    pub filtered_notes: Vec<usize>, // Indices of notes that match search
    pub list_state: ListState,
    pub scroll_state: ScrollbarState,
}

impl App {
    pub fn new(notebook_path: String) -> Self {
        let mut app = Self {
            notebook_path,
            notes: Vec::new(),
            selected_note_index: None,
            note_content: String::new(),
            input_mode: InputMode::Normal,
            search_query: String::new(),
            create_title: String::new(),
            cursor_position: 0,
            editor_scroll: 0,
            last_save: None,
            status_message: "Press 'h' for help".to_string(),
            filtered_notes: Vec::new(),
            list_state: ListState::default(),
            scroll_state: ScrollbarState::default(),
        };
        
        app.init_notebook();
        app.load_notes();
        app.update_filtered_notes();
        
        app
    }

    fn init_notebook(&mut self) {
        if let Err(e) = notebook::init_notebook(Path::new(&self.notebook_path)) {
            self.status_message = format!("Failed to initialize notebook: {}", e);
        }
    }

    fn load_notes(&mut self) {
        match std::fs::read_to_string(Path::new(&self.notebook_path).join(".bunko").join("index.json")) {
            Ok(content) => {
                match serde_json::from_str::<Vec<Note>>(&content) {
                    Ok(notes) => {
                        self.notes = notes;
                        self.update_filtered_notes();
                    }
                    Err(e) => {
                        self.status_message = format!("Failed to parse notes: {}", e);
                    }
                }
            }
            Err(_) => {
                // No notes yet, that's fine
                self.notes.clear();
                self.update_filtered_notes();
            }
        }
    }

    fn update_filtered_notes(&mut self) {
        if self.search_query.is_empty() {
            self.filtered_notes = (0..self.notes.len()).collect();
        } else {
            self.filtered_notes = self.notes
                .iter()
                .enumerate()
                .filter(|(_, note)| {
                    note.title.to_lowercase().contains(&self.search_query.to_lowercase())
                })
                .map(|(i, _)| i)
                .collect();
        }
        
        // Update scroll state
        self.scroll_state = self.scroll_state.content_length(self.filtered_notes.len());
        
        // Adjust selection if needed
        if let Some(selected) = self.selected_note_index {
            if !self.filtered_notes.contains(&selected) || self.filtered_notes.is_empty() {
                self.selected_note_index = None;
                self.note_content.clear();
            }
        }
    }

    fn select_note(&mut self, filtered_index: usize) {
        if filtered_index < self.filtered_notes.len() {
            let note_index = self.filtered_notes[filtered_index];
            self.selected_note_index = Some(note_index);
            self.load_note_content(note_index);
            self.list_state.select(Some(filtered_index));
        }
    }

    fn load_note_content(&mut self, note_index: usize) {
        if note_index < self.notes.len() {
            let note = &self.notes[note_index];
            let note_path = Path::new(&self.notebook_path).join(&note.relative_path);
            
            match std::fs::read_to_string(&note_path) {
                Ok(content) => {
                    self.note_content = content;
                    self.cursor_position = 0;
                    self.editor_scroll = 0;
                }
                Err(e) => {
                    self.status_message = format!("Failed to load note: {}", e);
                }
            }
        }
    }

    fn save_current_note(&mut self) {
        if let Some(note_index) = self.selected_note_index {
            if note_index < self.notes.len() {
                let note = &self.notes[note_index];
                let note_path = Path::new(&self.notebook_path).join(&note.relative_path);
                
                match std::fs::write(&note_path, &self.note_content) {
                    Ok(_) => {
                        if let Err(e) = notebook::update_note(
                            Path::new(&self.notebook_path),
                            &note.id,
                            &self.note_content,
                        ) {
                            self.status_message = format!("Failed to update index: {}", e);
                        } else {
                            self.last_save = Some(Instant::now());
                            self.status_message = "Note saved".to_string();
                            self.load_notes(); // Reload to get updated timestamps
                        }
                    }
                    Err(e) => {
                        self.status_message = format!("Failed to save note: {}", e);
                    }
                }
            }
        }
    }

    fn create_note(&mut self) {
        let title = if self.create_title.is_empty() {
            format!("New Note {}", chrono::Utc::now().format("%Y-%m-%d %H:%M:%S"))
        } else {
            self.create_title.clone()
        };
        
        let content = format!("# {}\n\nStart writing...", title);
        
        match notebook::add_note(Path::new(&self.notebook_path), &title, &content) {
            Ok(_) => {
                self.load_notes();
                self.update_filtered_notes();
                
                // Select the new note (it should be the last one)
                if !self.notes.is_empty() {
                    let new_note_index = self.notes.len() - 1;
                    if let Some(filtered_pos) = self.filtered_notes.iter().position(|&i| i == new_note_index) {
                        self.select_note(filtered_pos);
                    }
                }
                
                self.status_message = format!("Created note: {}", title);
                self.create_title.clear();
                self.input_mode = InputMode::Normal;
            }
            Err(e) => {
                self.status_message = format!("Failed to create note: {}", e);
            }
        }
    }

    pub fn handle_key_event(&mut self, key: KeyCode) {
        match self.input_mode {
            InputMode::Normal => self.handle_normal_key(key),
            InputMode::Editing => self.handle_editing_key(key),
            InputMode::Searching => self.handle_searching_key(key),
            InputMode::Creating => self.handle_creating_key(key),
        }
    }

    fn handle_normal_key(&mut self, key: KeyCode) {
        match key {
            KeyCode::Char('q') => std::process::exit(0),
            KeyCode::Char('h') => {
                self.status_message = "Keys: j/k=move, Enter=select, e=edit, s=search, n=new, Ctrl+S=save, q=quit".to_string();
            }
            KeyCode::Char('j') | KeyCode::Down => {
                let selected = self.list_state.selected().unwrap_or(0);
                if selected + 1 < self.filtered_notes.len() {
                    self.select_note(selected + 1);
                }
            }
            KeyCode::Char('k') | KeyCode::Up => {
                let selected = self.list_state.selected().unwrap_or(0);
                if selected > 0 {
                    self.select_note(selected - 1);
                } else if !self.filtered_notes.is_empty() {
                    self.select_note(self.filtered_notes.len() - 1);
                }
            }
            KeyCode::Enter => {
                if let Some(selected) = self.list_state.selected() {
                    if selected < self.filtered_notes.len() {
                        self.select_note(selected);
                    }
                }
            }
            KeyCode::Char('e') => {
                if self.selected_note_index.is_some() {
                    self.input_mode = InputMode::Editing;
                    self.status_message = "Editing mode - Press Esc to exit, Ctrl+S to save".to_string();
                }
            }
            KeyCode::Char('s') => {
                self.input_mode = InputMode::Searching;
                self.search_query.clear();
                self.status_message = "Search mode - Type to filter notes, Enter to confirm, Esc to cancel".to_string();
            }
            KeyCode::Char('n') => {
                self.input_mode = InputMode::Creating;
                self.create_title.clear();
                self.status_message = "Create note - Enter title, Enter to create, Esc to cancel".to_string();
            }
            _ => {}
        }
    }

    fn handle_editing_key(&mut self, key: KeyCode) {
        match key {
            KeyCode::Esc => {
                self.input_mode = InputMode::Normal;
                self.status_message = "Normal mode".to_string();
            }
            KeyCode::Char(c) => {
                if self.cursor_position <= self.note_content.len() {
                    self.note_content.insert(self.cursor_position, c);
                    self.cursor_position += 1;
                }
            }
            KeyCode::Backspace => {
                if self.cursor_position > 0 {
                    self.cursor_position -= 1;
                    self.note_content.remove(self.cursor_position);
                }
            }
            KeyCode::Enter => {
                self.note_content.insert(self.cursor_position, '\n');
                self.cursor_position += 1;
            }
            KeyCode::Left => {
                if self.cursor_position > 0 {
                    self.cursor_position -= 1;
                }
            }
            KeyCode::Right => {
                if self.cursor_position < self.note_content.len() {
                    self.cursor_position += 1;
                }
            }
            _ => {}
        }
        
        // Handle Ctrl+S for save
        if matches!(key, KeyCode::Char('s')) {
            // This would need additional handling for Ctrl modifier
            // For now, we'll implement auto-save
        }
    }

    fn handle_searching_key(&mut self, key: KeyCode) {
        match key {
            KeyCode::Esc => {
                self.input_mode = InputMode::Normal;
                self.search_query.clear();
                self.update_filtered_notes();
                self.status_message = "Search cancelled".to_string();
            }
            KeyCode::Enter => {
                self.input_mode = InputMode::Normal;
                self.update_filtered_notes();
                if !self.filtered_notes.is_empty() {
                    self.select_note(0);
                }
                self.status_message = format!("Found {} notes", self.filtered_notes.len());
            }
            KeyCode::Char(c) => {
                self.search_query.push(c);
                self.update_filtered_notes();
            }
            KeyCode::Backspace => {
                self.search_query.pop();
                self.update_filtered_notes();
            }
            _ => {}
        }
    }

    fn handle_creating_key(&mut self, key: KeyCode) {
        match key {
            KeyCode::Esc => {
                self.input_mode = InputMode::Normal;
                self.create_title.clear();
                self.status_message = "Create cancelled".to_string();
            }
            KeyCode::Enter => {
                self.create_note();
            }
            KeyCode::Char(c) => {
                self.create_title.push(c);
            }
            KeyCode::Backspace => {
                self.create_title.pop();
            }
            _ => {}
        }
    }
}

pub fn run_tui(notebook_path: &str) -> Result<(), Box<dyn Error>> {
    // Setup terminal
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    // Create app and run
    let app = App::new(notebook_path.to_string());
    let res = run_app(&mut terminal, app);

    // Restore terminal
    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;

    if let Err(err) = res {
        println!("{:?}", err);
    }

    Ok(())
}

fn run_app<B: Backend>(terminal: &mut Terminal<B>, mut app: App) -> io::Result<()> {
    loop {
        terminal.draw(|f| ui(f, &mut app))?;

        if event::poll(Duration::from_millis(250))? {
            if let Event::Key(key) = event::read()? {
                if key.kind == KeyEventKind::Press {
                    app.handle_key_event(key.code);
                }
            }
        }
    }
}

fn ui(f: &mut Frame, app: &mut App) {
    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(30), Constraint::Percentage(70)])
        .split(f.area());

    // Left panel - notes list
    let notes_list: Vec<ListItem> = app
        .filtered_notes
        .iter()
        .map(|&i| {
            let note = &app.notes[i];
            let style = if Some(i) == app.selected_note_index {
                Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)
            } else {
                Style::default().fg(Color::White)
            };
            
            ListItem::new(Line::from(vec![
                Span::styled(&note.title, style),
            ]))
        })
        .collect();

    let notes_block = Block::default()
        .borders(Borders::ALL)
        .title(format!(" Notes ({}) ", app.filtered_notes.len()));

    let notes_widget = List::new(notes_list)
        .block(notes_block)
        .highlight_style(Style::default().fg(Color::Black).bg(Color::Yellow));

    f.render_stateful_widget(notes_widget, chunks[0], &mut app.list_state);

    // Right panel - note content or editor
    let right_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Min(3), Constraint::Length(3)])
        .split(chunks[1]);

    // Note content area
    if let Some(note_index) = app.selected_note_index {
        if note_index < app.notes.len() {
            let note = &app.notes[note_index];
            let title = match &app.input_mode {
                InputMode::Editing => format!(" {} (EDITING) ", note.title),
                _ => format!(" {} ", note.title),
            };

            let content_block = Block::default()
                .borders(Borders::ALL)
                .title(title);

            let content = if matches!(app.input_mode, InputMode::Editing) {
                // Show cursor in editing mode
                let mut display_content = app.note_content.clone();
                if app.cursor_position <= display_content.len() {
                    display_content.insert(app.cursor_position, '|');
                }
                Text::from(display_content)
            } else {
                Text::from(app.note_content.clone())
            };

            let paragraph = Paragraph::new(content)
                .block(content_block)
                .wrap(Wrap { trim: true })
                .scroll((app.editor_scroll as u16, 0));

            f.render_widget(paragraph, right_chunks[0]);
        }
    } else {
        let help_block = Block::default()
            .borders(Borders::ALL)
            .title(" Welcome to Bunko TUI ");

        let help_text = Text::from(vec![
            Line::from(""),
            Line::from("No note selected. Use the following keys:"),
            Line::from(""),
            Line::from("j/k or ↑/↓  - Navigate notes"),
            Line::from("Enter       - Select note"),
            Line::from("e           - Edit selected note"),
            Line::from("s           - Search notes"),
            Line::from("n           - Create new note"),
            Line::from("h           - Show help"),
            Line::from("q           - Quit"),
        ]);

        let paragraph = Paragraph::new(help_text)
            .block(help_block)
            .alignment(Alignment::Left);

        f.render_widget(paragraph, right_chunks[0]);
    }

    // Status bar
    let status_text = match &app.input_mode {
        InputMode::Searching => format!("Search: {} | {}", app.search_query, app.status_message),
        InputMode::Creating => format!("Title: {} | {}", app.create_title, app.status_message),
        _ => app.status_message.clone(),
    };

    let status_block = Block::default()
        .borders(Borders::ALL);

    let status_paragraph = Paragraph::new(status_text)
        .block(status_block);

    f.render_widget(status_paragraph, right_chunks[1]);
}