use crate::cli::{load_manager_state, save_manager_state};
use crate::process::ProcessManager;
use crate::script::ScriptDetector;
use crate::models::Script;
use anyhow::Result;
use chrono::Local;
use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{
    backend::CrosstermBackend,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::Line,
    widgets::{Block, Borders, List, ListItem, ListState, Paragraph},
    Frame, Terminal,
};
use std::env;
use std::io;
use std::time::{Duration, Instant};

#[derive(PartialEq)]
enum Section {
    Processes,
    Scripts,
}

pub struct App {
    manager: ProcessManager,
    scripts: Vec<Script>,
    selected_section: Section,
    process_list_state: ListState,
    script_list_state: ListState,
    last_update: Instant,
    should_quit: bool,
}

impl App {
    pub fn new() -> Result<Self> {
        let manager = load_manager_state()?;
        let cwd = env::current_dir()?;
        let scripts = ScriptDetector::detect_scripts(&cwd).unwrap_or_default();

        let mut app = Self {
            manager,
            scripts,
            selected_section: Section::Processes,
            process_list_state: ListState::default(),
            script_list_state: ListState::default(),
            last_update: Instant::now(),
            should_quit: false,
        };

        // Select first item by default
        if !app.manager.list_processes().is_empty() {
            app.process_list_state.select(Some(0));
        } else if !app.scripts.is_empty() {
            app.selected_section = Section::Scripts;
            app.script_list_state.select(Some(0));
        }

        Ok(app)
    }

    pub fn run(&mut self) -> Result<()> {
        // Setup terminal
        enable_raw_mode()?;
        let mut stdout = io::stdout();
        execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
        let backend = CrosstermBackend::new(stdout);
        let mut terminal = Terminal::new(backend)?;

        let res = self.run_app(&mut terminal);

        // Restore terminal
        disable_raw_mode()?;
        execute!(
            terminal.backend_mut(),
            LeaveAlternateScreen,
            DisableMouseCapture
        )?;
        terminal.show_cursor()?;

        if let Err(err) = res {
            eprintln!("Error: {:?}", err);
        }

        Ok(())
    }

    fn run_app(&mut self, terminal: &mut Terminal<CrosstermBackend<io::Stdout>>) -> Result<()> {
        loop {
            terminal.draw(|f| self.ui(f))?;

            // Auto-refresh every second
            if self.last_update.elapsed() >= Duration::from_secs(1) {
                self.manager.refresh();
                self.last_update = Instant::now();
            }

            // Handle input (with timeout for refresh)
            if event::poll(Duration::from_millis(100))? {
                if let Event::Key(key) = event::read()? {
                    match key.code {
                        KeyCode::Char('q') => {
                            self.should_quit = true;
                        }
                        KeyCode::Tab => {
                            self.switch_section();
                        }
                        KeyCode::Char('j') | KeyCode::Down => {
                            self.next_item();
                        }
                        KeyCode::Char('k') | KeyCode::Up => {
                            self.previous_item();
                        }
                        KeyCode::Char('d') => {
                            self.kill_selected_process()?;
                        }
                        KeyCode::Enter => {
                            self.run_selected_script()?;
                        }
                        _ => {}
                    }
                }
            }

            if self.should_quit {
                break;
            }
        }

        Ok(())
    }

    fn ui(&mut self, f: &mut Frame) {
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(3),
                Constraint::Min(10),
                Constraint::Min(10),
                Constraint::Length(3),
            ])
            .split(f.area());

        // Title
        let title = Paragraph::new("devproc - Development Process Manager")
            .style(Style::default().fg(Color::Cyan))
            .block(Block::default().borders(Borders::ALL));
        f.render_widget(title, chunks[0]);

        // Processes section
        self.render_processes(f, chunks[1]);

        // Scripts section
        self.render_scripts(f, chunks[2]);

        // Help
        let help = Paragraph::new("[j/k] move [Tab] switch [Enter] run [d] kill [q] quit")
            .style(Style::default().fg(Color::DarkGray))
            .block(Block::default().borders(Borders::ALL));
        f.render_widget(help, chunks[3]);
    }

    fn render_processes(&mut self, f: &mut Frame, area: Rect) {
        let is_focused = self.selected_section == Section::Processes;
        let border_color = if is_focused { Color::Blue } else { Color::DarkGray };

        let processes = self.manager.list_processes();
        let items: Vec<ListItem> = processes
            .iter()
            .map(|p| {
                let metrics = self.manager.get_metrics(p).unwrap_or_default();
                let memory_mb = metrics.memory_bytes / 1024 / 1024;
                let port_str = p.port.map(|p| format!("{}", p)).unwrap_or_else(|| "-".to_string());

                let duration = Local::now().signed_duration_since(p.started_at);
                let started = if duration.num_hours() > 0 {
                    format!("{}h", duration.num_hours())
                } else if duration.num_minutes() > 0 {
                    format!("{}m", duration.num_minutes())
                } else {
                    format!("{}s", duration.num_seconds())
                };

                let content = format!(
                    "{:<25} {:>5.1}% {:>6}MB  :{:<5}  {}",
                    p.name,
                    metrics.cpu_percent,
                    memory_mb,
                    port_str,
                    started
                );

                ListItem::new(Line::from(content))
            })
            .collect();

        let list = List::new(items)
            .block(
                Block::default()
                    .title(format!(" Running Processes ({}) ", processes.len()))
                    .borders(Borders::ALL)
                    .border_style(Style::default().fg(border_color)),
            )
            .highlight_style(
                Style::default()
                    .bg(Color::DarkGray)
                    .add_modifier(Modifier::BOLD),
            )
            .highlight_symbol("▶ ");

        f.render_stateful_widget(list, area, &mut self.process_list_state);
    }

    fn render_scripts(&mut self, f: &mut Frame, area: Rect) {
        let is_focused = self.selected_section == Section::Scripts;
        let border_color = if is_focused { Color::Blue } else { Color::DarkGray };

        let items: Vec<ListItem> = self
            .scripts
            .iter()
            .map(|s| {
                let content = format!("{:<20} {}", s.name, s.command);
                ListItem::new(Line::from(content))
            })
            .collect();

        let cwd = env::current_dir().unwrap_or_default();
        let cwd_str = cwd.to_string_lossy();

        let list = List::new(items)
            .block(
                Block::default()
                    .title(format!(" Scripts - {} ", cwd_str))
                    .borders(Borders::ALL)
                    .border_style(Style::default().fg(border_color)),
            )
            .highlight_style(
                Style::default()
                    .bg(Color::DarkGray)
                    .add_modifier(Modifier::BOLD),
            )
            .highlight_symbol("▶ ");

        f.render_stateful_widget(list, area, &mut self.script_list_state);
    }

    fn switch_section(&mut self) {
        self.selected_section = match self.selected_section {
            Section::Processes => {
                if !self.scripts.is_empty() {
                    if self.script_list_state.selected().is_none() {
                        self.script_list_state.select(Some(0));
                    }
                    Section::Scripts
                } else {
                    Section::Processes
                }
            }
            Section::Scripts => {
                let processes = self.manager.list_processes();
                if !processes.is_empty() {
                    if self.process_list_state.selected().is_none() {
                        self.process_list_state.select(Some(0));
                    }
                    Section::Processes
                } else {
                    Section::Scripts
                }
            }
        };
    }

    fn next_item(&mut self) {
        match self.selected_section {
            Section::Processes => {
                let len = self.manager.list_processes().len();
                if len == 0 {
                    return;
                }
                let i = match self.process_list_state.selected() {
                    Some(i) => (i + 1) % len,
                    None => 0,
                };
                self.process_list_state.select(Some(i));
            }
            Section::Scripts => {
                let len = self.scripts.len();
                if len == 0 {
                    return;
                }
                let i = match self.script_list_state.selected() {
                    Some(i) => (i + 1) % len,
                    None => 0,
                };
                self.script_list_state.select(Some(i));
            }
        }
    }

    fn previous_item(&mut self) {
        match self.selected_section {
            Section::Processes => {
                let len = self.manager.list_processes().len();
                if len == 0 {
                    return;
                }
                let i = match self.process_list_state.selected() {
                    Some(i) => {
                        if i == 0 {
                            len - 1
                        } else {
                            i - 1
                        }
                    }
                    None => 0,
                };
                self.process_list_state.select(Some(i));
            }
            Section::Scripts => {
                let len = self.scripts.len();
                if len == 0 {
                    return;
                }
                let i = match self.script_list_state.selected() {
                    Some(i) => {
                        if i == 0 {
                            len - 1
                        } else {
                            i - 1
                        }
                    }
                    None => 0,
                };
                self.script_list_state.select(Some(i));
            }
        }
    }

    fn kill_selected_process(&mut self) -> Result<()> {
        if self.selected_section != Section::Processes {
            return Ok(());
        }

        if let Some(idx) = self.process_list_state.selected() {
            let processes = self.manager.list_processes();
            if let Some(process) = processes.get(idx) {
                let id = process.id;
                self.manager.kill_process(&id, 15)?;
                save_manager_state(&self.manager)?;

                // Adjust selection
                let new_len = self.manager.list_processes().len();
                if new_len == 0 {
                    self.process_list_state.select(None);
                } else if idx >= new_len {
                    self.process_list_state.select(Some(new_len - 1));
                }
            }
        }

        Ok(())
    }

    fn run_selected_script(&mut self) -> Result<()> {
        if self.selected_section != Section::Scripts {
            return Ok(());
        }

        if let Some(idx) = self.script_list_state.selected() {
            if let Some(script) = self.scripts.get(idx) {
                let process = crate::script::ScriptRunner::run_script(
                    script,
                    None,
                    None,
                    Vec::new(),
                )?;
                self.manager.add_process(process);
                save_manager_state(&self.manager)?;
            }
        }

        Ok(())
    }
}
