use std::io::{self, Result};

use std::env::{self, args, set_current_dir};
use std::fs::{self, read_dir, DirEntry, FileType, ReadDir};
use std::os::unix::fs::FileTypeExt;
use std::path::{Path, PathBuf};

use ratatui::{
    buffer::Buffer,
    crossterm::event::{self, Event, KeyCode, KeyEvent, KeyEventKind},
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::Stylize,
    symbols::border,
    text::{Line, Text},
    widgets::{
        block::{Position, Title},
        Block, List, ListDirection, Paragraph, Widget,
    },
    Frame,
};

mod tui;

fn main() -> Result<()> {
    let mut terminal = tui::init()?;

    match args().len() {
        1 => (),
        2 => set_current_dir(args().skip(1).next().unwrap())?,
        _ => {
            println!("too many arguments");
            return Ok(());
        }
    }
    let mut app = App::new(
        CurrentDir {
            value: env::current_dir()?,
        },
        Content {
            value: read_dir(env::current_dir().unwrap())
                .unwrap()
                .collect::<Result<Vec<DirEntry>>>()
                .unwrap(),
        },
        Status {
            value: "status".to_string(),
        },
        Command {
            value: ":command".to_string(),
        },
        false,
    );

    let app_result = app.run(&mut terminal);
    tui::restore()?;
    app_result
}

pub struct App {
    current_dir: CurrentDir,
    content: Content,
    status: Status,
    command: Command,
    exit: bool,
}

impl App {
    pub fn new(
        current_dir: CurrentDir,
        content: Content,
        status: Status,
        command: Command,
        exit: bool,
    ) -> App {
        App {
            current_dir,
            content,
            status,
            command,
            exit,
        }
    }

    pub fn run(&mut self, terminal: &mut tui::Tui) -> io::Result<()> {
        while !self.exit {
            terminal.draw(|frame| self.render_frame(frame))?;
            self.handle_events()?;
        }
        Ok(())
    }

    fn render_frame(&self, frame: &mut Frame) {
        let areas = Layout::new(
            Direction::Vertical,
            [
                Constraint::Length(1),
                Constraint::Fill(1),
                Constraint::Length(1),
                Constraint::Length(1),
            ],
        )
        .split(frame.size());

        frame.render_widget(&self.current_dir, areas[0]);
        frame.render_widget(&self.content, areas[1]);
        frame.render_widget(&self.status, areas[2]);
        frame.render_widget(&self.command, areas[3]);
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
        match key_event.code {
            KeyCode::Char('q') => self.exit(),
            KeyCode::Char('.') => self.change_dir(),
            _ => {}
        }
    }

    fn exit(&mut self) {
        self.exit = true;
    }

    fn change_dir(&mut self) {
        set_current_dir("..").unwrap();
        self.current_dir.value = env::current_dir().unwrap();
    }
}

pub struct CurrentDir {
    value: PathBuf,
}

impl Widget for &CurrentDir {
    fn render(self, area: Rect, buf: &mut Buffer) {
        Paragraph::new(self.value.to_str().unwrap()).render(area, buf);
    }
}

pub struct Content {
    value: Vec<DirEntry>,
}

impl Widget for &Content {
    fn render(self, area: Rect, buf: &mut Buffer) {
        /*
        List::new(
            self.value
                .into_iter()
                .map(|a| a.file_name().into_string().unwrap()),
        )
        */
        List::new((1..10).map(|a| a.to_string()))
            //.style(Style::default().fg(Color::White))
            // .highlight_style(Style::default().add_modifier(Modifier::ITALIC))
            // .highlight_symbol(">>")
            // .repeat_highlight_symbol(true)
            .direction(ListDirection::TopToBottom)
            .render(area, buf);
    }
}

pub struct Status {
    value: String,
}

impl Widget for &Status {
    fn render(self, area: Rect, buf: &mut Buffer) {
        Paragraph::new(self.value.clone()).render(area, buf);
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
