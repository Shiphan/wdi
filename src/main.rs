use core::fmt;
use std::env::{self, args, set_current_dir};
use std::fs::{read_dir, DirEntry};
use std::io::{self, stderr, Result, Stderr};
use std::path::PathBuf;
use std::process::ExitCode;
use std::str::FromStr;

use ratatui::{
    backend::CrosstermBackend,
    buffer::Buffer,
    crossterm::{
        event::{self, Event, KeyCode, KeyEvent, KeyEventKind},
        execute,
        terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
    },
    layout::{Constraint, Layout, Rect},
    style::{Color, Style},
    text::Text,
    widgets::{List, ListDirection, ListState, Paragraph, StatefulWidget, Widget},
    Frame, Terminal,
};

fn main() -> Result<ExitCode> {
    match args().len() {
        1 => (),
        2 => {
            let arg1 = args().skip(1).next().unwrap_or("".to_string());
            if arg1 != "" {
                let _ = set_current_dir(arg1);
            }
        }
        _ => {
            println!("too many arguments");
            return Ok(ExitCode::from(1));
        }
    }

    execute!(stderr(), EnterAlternateScreen)?;
    enable_raw_mode()?;

    let mut terminal = Terminal::new(CrosstermBackend::new(stderr()))?;
    let mut app = App::new(
        Content {
            value: read_dir(env::current_dir()?)?.collect::<Result<Vec<DirEntry>>>()?,
            keyword: None,
            targets: vec![],
            state: ListState::default(),
        },
        Status {
            mode: Mode::Normal,
            current_dir: env::current_dir()?,
        },
        Command {
            value: "".to_string(),
        },
        false,
        PathBuf::new(),
    );

    let app_result = app.run(&mut terminal);

    execute!(stderr(), LeaveAlternateScreen)?;
    disable_raw_mode()?;

    app_result?;
    print!("{}", app.exit_path.to_str().unwrap());

    Ok(ExitCode::from(0))
}

struct App {
    content: Content,
    status: Status,
    command: Command,
    exit: bool,
    exit_path: PathBuf,
}

impl App {
    fn new(
        content: Content,
        status: Status,
        command: Command,
        exit: bool,
        exit_path: PathBuf,
    ) -> App {
        App {
            content,
            status,
            command,
            exit,
            exit_path,
        }
    }

    fn run(&mut self, terminal: &mut Terminal<CrosstermBackend<Stderr>>) -> io::Result<()> {
        self.content.state.select_first();

        while !self.exit {
            terminal.draw(|frame| self.render_frame(frame))?;
            self.handle_events()?;
        }
        Ok(())
    }

    fn render_frame(&mut self, frame: &mut Frame) {
        let areas = Layout::vertical([
            Constraint::Fill(1),
            Constraint::Length(1),
            Constraint::Length(1),
        ])
        .split(frame.size());

        frame.render_widget(&mut self.content, areas[0]);
        frame.render_widget(&self.status, areas[1]);
        frame.render_widget(&self.command, areas[2]);
    }

    fn handle_events(&mut self) -> io::Result<()> {
        match event::read()? {
            Event::Key(key_event) if key_event.kind == KeyEventKind::Press => {
                self.handle_key_event(key_event)
            }
            _ => {}
        };
        Ok(())
    }

    fn handle_key_event(&mut self, key_event: KeyEvent) {
        match self.status.mode {
            Mode::Normal => match key_event.code {
                KeyCode::Char('w') => {
                    self.exit_path = self.status.current_dir.clone();
                    self.exit();
                }
                KeyCode::Char('q') => {
                    self.exit_path = PathBuf::from_str(".").unwrap();
                    self.exit();
                }
                KeyCode::Char('j') | KeyCode::Down => self.content.down(),
                KeyCode::Char('k') | KeyCode::Up => self.content.up(),
                KeyCode::Char('/') => {
                    self.status.mode = Mode::Command;
                    self.content.clear_search();
                    self.command.value = "/".to_string();
                }
                KeyCode::Char('n') => self.content.select_next_target(),
                KeyCode::Char('N') => self.content.select_previous_target(),
                KeyCode::Enter => {
                    self.content.enter();
                    self.status.update_current_dir();
                    self.content.update();
                }
                KeyCode::Esc => {
                    if self.content.keyword != None {
                        self.content.clear_search();
                        self.command.value = "".to_string();
                    }
                }
                _ => (),
            },
            Mode::Command => match key_event.code {
                KeyCode::Char(c) => {
                    let mut keyword = self.content.keyword.clone().unwrap_or("".to_string());
                    keyword.push(c);
                    self.content.keyword = Some(keyword);
                    self.content
                        .search(self.content.keyword.clone().unwrap_or("".to_string()));
                    self.command.value = format!(
                        "/{}",
                        self.content.keyword.clone().unwrap_or("".to_string())
                    );
                }
                KeyCode::Backspace => match &self.content.keyword {
                    Some(a) => match a.as_str() {
                        "" => {
                            self.content.clear_search();
                            self.command.value = "".to_string();
                            self.status.mode = Mode::Normal;
                        }
                        _ => {
                            let mut keyword = a.clone();
                            keyword.pop();
                            self.content.search(keyword);
                            self.command.value = format!(
                                "/{}",
                                self.content.keyword.clone().unwrap_or("".to_string())
                            );
                        }
                    },
                    None => {
                        self.content.clear_search();
                        self.command.value = "".to_string();
                        self.status.mode = Mode::Normal;
                    }
                },
                KeyCode::Enter => {
                    self.status.mode = Mode::Normal;
                }
                KeyCode::Esc => {
                    self.content.clear_search();
                    self.command.value = "".to_string();
                    self.status.mode = Mode::Normal;
                }
                _ => (),
            },
        }
    }

    fn exit(&mut self) {
        self.exit = true;
    }
}

struct Content {
    value: Vec<DirEntry>,
    keyword: Option<String>,
    targets: Vec<usize>,
    state: ListState,
}

impl Content {
    fn update(&mut self) {
        self.value = read_dir(env::current_dir().unwrap())
            .unwrap()
            .collect::<Result<Vec<DirEntry>>>()
            .unwrap();
    }
    fn search(&mut self, keyword: String) {
        self.keyword = Some(keyword.clone());
        self.targets = self
            .value
            .iter()
            .enumerate()
            .filter(|&(_, v)| {
                v.file_name()
                    .into_string()
                    .unwrap()
                    .to_lowercase()
                    .contains(&keyword.to_lowercase())
            })
            .map(|(i, _)| i + 2)
            .collect();
    }
    fn select_next_target(&mut self) {
        match self.targets.len() {
            0 => (),
            _ => {
                let current_selected = self.state.selected().unwrap_or(0);
                for target in self.targets.clone() {
                    if target > current_selected {
                        self.state.select(Some(target));
                        return;
                    }
                }
                self.state.select(Some(self.targets[0]));
            }
        }
    }
    fn select_previous_target(&mut self) {
        match self.targets.len() {
            0 => (),
            _ => {
                let current_selected = self.state.selected().unwrap_or(0);
                let mut targets = self.targets.clone();
                targets.reverse();
                for target in &targets {
                    if target < &current_selected {
                        self.state.select(Some(*target));
                        return;
                    }
                }
                self.state.select(Some(targets[0]));
            }
        }
    }
    fn clear_search(&mut self) {
        self.targets = vec![];
        self.keyword = None;
    }
    fn up(&mut self) {
        self.state.select_previous();
        self.update();
    }
    fn down(&mut self) {
        self.state.select_next();
        self.update();
    }
    fn enter(&mut self) {
        match self.state.selected() {
            Some(i) => match i {
                0 => {
                    let _ = set_current_dir("..");
                    self.state.select_first();
                }
                1 => self.update(),
                _ => {
                    let _ = set_current_dir(self.value[i - 2].path());
                    self.state.select_first();
                }
            },
            None => (),
        }
    }
}

impl Widget for &mut Content {
    fn render(self, area: Rect, buf: &mut Buffer) {
        StatefulWidget::render(
            List::new(
                [
                    vec!["../".to_string(), "./".to_string()],
                    self.value
                        .iter()
                        .map(|a| {
                            if a.file_type().unwrap().is_dir() {
                                a.file_name().into_string().unwrap() + "/"
                            } else {
                                a.file_name().into_string().unwrap()
                            }
                        })
                        .collect::<Vec<String>>(),
                ]
                .concat()
                .iter()
                .enumerate()
                .map(|(i, v)| {
                    if self.targets.contains(&i) {
                        Text::styled(v, Style::new().bg(Color::DarkGray))
                    } else {
                        Text::styled(v, Style::new())
                    }
                })
                .collect::<Vec<Text>>(),
            )
            .highlight_style(Style::new().fg(Color::DarkGray).bg(Color::White))
            .direction(ListDirection::TopToBottom),
            area,
            buf,
            &mut self.state,
        );
    }
}

enum Mode {
    Normal,
    Command,
}

impl fmt::Display for Mode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Mode::Normal => write!(f, "Normal"),
            Mode::Command => write!(f, "Command"),
        }
    }
}

struct Status {
    mode: Mode,
    current_dir: PathBuf,
}

impl Status {
    fn update_current_dir(&mut self) {
        self.current_dir = env::current_dir().unwrap();
    }
}

impl Widget for &Status {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let areas = Layout::horizontal([
            Constraint::Length((self.mode.to_string().len() + 2) as u16),
            Constraint::Fill(1),
        ])
        .split(area);

        Paragraph::new(format!(" {} ", self.mode.to_string()))
            .style(Style::new().fg(Color::Black).bg(match self.mode {
                Mode::Normal => Color::Blue,
                Mode::Command => Color::Yellow,
            }))
            .render(areas[0], buf);
        Paragraph::new(format!(" {} ", self.current_dir.to_str().unwrap())).render(areas[1], buf);
    }
}

struct Command {
    value: String,
}

impl Widget for &Command {
    fn render(self, area: Rect, buf: &mut Buffer) {
        Paragraph::new(self.value.clone()).render(area, buf);
    }
}
