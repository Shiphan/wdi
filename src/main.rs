use core::fmt;
use std::io::{self, stderr, Result, Stderr};
use std::process::ExitCode;

use std::env::{self, args, set_current_dir};
use std::fs::{read_dir, DirEntry};
use std::path::PathBuf;
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

pub struct App {
    content: Content,
    status: Status,
    command: Command,
    exit: bool,
    exit_path: PathBuf,
}

impl App {
    pub fn new(
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

    pub fn run(&mut self, terminal: &mut Terminal<CrosstermBackend<Stderr>>) -> io::Result<()> {
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
                KeyCode::Enter => {
                    self.content.enter();
                    self.status.update_current_dir();
                    self.content.update();
                }
                _ => (),
            },
            Mode::Command => self.status.mode = Mode::Normal,
        }
    }

    fn exit(&mut self) {
        self.exit = true;
    }
}

pub struct Content {
    value: Vec<DirEntry>,
    state: ListState,
}

impl Content {
    fn update(&mut self) {
        self.value = read_dir(env::current_dir().unwrap())
            .unwrap()
            .collect::<Result<Vec<DirEntry>>>()
            .unwrap();
    }
    fn up(&mut self) {
        let i = match self.state.selected() {
            Some(i) => {
                if i == 0 {
                    0
                } else {
                    i - 1
                }
            }
            None => 0,
        };
        self.state.select(Some(i));
        self.update();
    }
    fn down(&mut self) {
        let i = match self.state.selected() {
            Some(i) => {
                if i >= self.value.len() + 1 {
                    i
                } else {
                    i + 1
                }
            }
            None => 0,
        };
        self.state.select(Some(i));
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
                .concat(),
            )
            .highlight_symbol(">>")
            .direction(ListDirection::TopToBottom),
            //.style(Style::default().fg(Color::White))
            // .highlight_style(Style::default().add_modifier(Modifier::ITALIC))
            // .repeat_highlight_symbol(true)
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

pub struct Status {
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
            .style(Style::new().bg(Color::Blue))
            .render(areas[0], buf);
        Paragraph::new(self.current_dir.to_str().unwrap()).render(areas[1], buf);
    }
}

pub struct Command {
    value: String,
}

impl Widget for &Command {
    fn render(self, area: Rect, buf: &mut Buffer) {
        Paragraph::new(self.value.clone()).render(area, buf);
    }
}

/*
enum EntryType {
    Dir,
    File,
    Symlink,
    Socket,
    Fifo,
    CharDevice,
    BlockDevice,
    None,
}

fn what_entry_type(file_type: &FileType) -> EntryType {
    if file_type.is_dir() {
        EntryType::Dir
    } else if file_type.is_file() {
        EntryType::File
    } else if file_type.is_symlink() {
        EntryType::Symlink
    } else if file_type.is_socket() {
        EntryType::Socket
    } else if file_type.is_fifo() {
        EntryType::Fifo
    } else if file_type.is_char_device() {
        EntryType::CharDevice
    } else if file_type.is_block_device() {
        EntryType::BlockDevice
    } else {
        EntryType::None
    }
}
*/
