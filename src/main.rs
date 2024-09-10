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
    let message: Option<String> = match args().len() {
        1 => None,
        2 => {
            let arg1 = args().skip(1).next().unwrap_or(String::from(""));
            if arg1 != "" {
                match set_current_dir(&arg1) {
                    Ok(_) => None,
                    Err(err) => Some(format!(
                        "Unable to change directory to `{arg1}`. (Error: {err})",
                    )),
                }
            } else {
                None
            }
        }
        _ => {
            println!("too many arguments");
            return Ok(ExitCode::from(1));
        }
    };

    execute!(stderr(), EnterAlternateScreen)?;
    enable_raw_mode()?;

    let mut terminal = Terminal::new(CrosstermBackend::new(stderr()))?;
    let mut app = App::new(
        Content {
            value: read_dir(env::current_dir()?)?.collect::<Result<Vec<DirEntry>>>()?,
            keyword: None,
            targets: vec![],
            recover_point: None,
            state: ListState::default(),
        },
        Status {
            mode: Mode::Normal,
            current_dir: env::current_dir()?,
            ruler: "1".to_string(),
        },
        match message {
            Some(a) => Command {
                value: a,
                style: Style::new().fg(Color::Red),
            },
            None => Command {
                value: String::from(""),
                style: Style::new(),
            },
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
                KeyCode::Char('j') | KeyCode::Down => {
                    self.content.down();
                    self.status.update_ruler(&self.content);
                }
                KeyCode::Char('k') | KeyCode::Up => {
                    self.content.up();
                    self.status.update_ruler(&self.content);
                }
                KeyCode::Char('/') => {
                    self.status.mode = Mode::Command;
                    self.content.clear_search();
                    self.content.set_recover_point();
                    self.command.reset();
                    self.command.value = "/".to_string();
                }
                KeyCode::Char('n') => {
                    self.content.select_next_target();
                    self.status.update_ruler(&self.content);
                }
                KeyCode::Char('N') => {
                    self.content.select_previous_target();
                    self.status.update_ruler(&self.content);
                }
                KeyCode::Enter => {
                    match self.content.enter() {
                        Ok(_) => (),
                        Err(err) => {
                            self.command.value = format!("Error: {err}");
                            self.command.style = Style::new().fg(Color::Red);
                        }
                    };
                    self.status.update_current_dir();
                    self.status.update_ruler(&self.content);
                }
                KeyCode::Esc => {
                    if self.content.keyword != None {
                        self.content.clear_search();
                        self.command.reset();
                        self.command.value = "".to_string();
                    }
                }
                _ => (),
            },
            Mode::Command => match key_event.code {
                KeyCode::Char(c) => {
                    let mut keyword = self.content.keyword.clone().unwrap_or("".to_string());
                    keyword.push(c);
                    self.content.keyword = Some(keyword.clone());
                    self.content.search(keyword);

                    self.command.value = format!(
                        "/{}",
                        self.content.keyword.clone().unwrap_or("".to_string())
                    );
                    self.content.temporary_select_next_target();
                    self.status.update_ruler(&self.content);
                }
                KeyCode::Backspace => match &self.content.keyword {
                    Some(a) => match a.as_str() {
                        "" => {
                            self.content.clear_search();
                            self.command.value = "".to_string();
                            self.status.mode = Mode::Normal;
                            self.content.recover_selection();
                            self.content.clear_recover_point();
                            self.status.update_ruler(&self.content);
                        }
                        _ => {
                            let mut keyword = a.clone();
                            keyword.pop();
                            self.content.search(keyword);

                            self.command.value = format!(
                                "/{}",
                                self.content.keyword.clone().unwrap_or("".to_string())
                            );
                            self.content.recover_selection();
                            self.content.temporary_select_next_target();
                            self.status.update_ruler(&self.content);
                        }
                    },
                    None => {
                        self.content.clear_search();
                        self.command.value = "".to_string();
                        self.status.mode = Mode::Normal;
                        self.content.recover_selection();
                        self.content.clear_recover_point();
                        self.status.update_ruler(&self.content);
                    }
                },
                KeyCode::Enter => {
                    self.status.mode = Mode::Normal;
                }
                KeyCode::Esc => {
                    self.content.clear_search();
                    self.command.value = "".to_string();
                    self.status.mode = Mode::Normal;
                    self.content.recover_selection();
                    self.content.clear_recover_point();
                    self.status.update_ruler(&self.content);
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
    recover_point: Option<usize>,
    state: ListState,
}

impl Content {
    fn update(&mut self) -> Result<()> {
        self.value = read_dir(env::current_dir().unwrap())?.collect::<Result<Vec<DirEntry>>>()?;
        match self.keyword.clone() {
            Some(keyword) => self.search(keyword),
            None => (),
        }
        Ok(())
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
    fn set_recover_point(&mut self) {
        self.recover_point = self.state.selected();
    }
    fn clear_recover_point(&mut self) {
        self.recover_point = None;
    }
    fn temporary_select_next_target(&mut self) {
        match self.targets.len() {
            0 => self.recover_selection(),
            _ => {
                let point = self
                    .recover_point
                    .unwrap_or(self.state.selected().unwrap_or(0));
                for &target in &self.targets {
                    if target >= point {
                        self.state.select(Some(target));
                        return;
                    }
                }
                self.state.select(Some(self.targets[0]));
            }
        }
    }
    fn recover_selection(&mut self) {
        match self.recover_point {
            Some(a) => {
                self.state.select(Some(a));
            }
            None => (),
        }
    }
    fn select_previous_target(&mut self) {
        match self.targets.len() {
            0 => (),
            _ => {
                let current_selected = self.state.selected().unwrap_or(0);
                let mut targets = self.targets.clone();
                targets.reverse();
                for &target in &targets {
                    if target < current_selected {
                        self.state.select(Some(target));
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
    }
    fn down(&mut self) {
        self.state.select_next();
    }
    fn enter(&mut self) -> Result<()> {
        match self.state.selected() {
            Some(i) => match i {
                0 => {
                    set_current_dir("..")?;
                    self.update()?;
                    self.state.select_first();
                    Ok(())
                }
                1 => {
                    self.update()?;
                    Ok(())
                }
                _ => {
                    set_current_dir(self.value[i - 2].path())?;
                    self.update()?;
                    self.state.select_first();
                    Ok(())
                }
            },
            None => Ok(()),
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
    ruler: String,
}

impl Status {
    fn update_current_dir(&mut self) {
        self.current_dir = env::current_dir().unwrap();
    }
    fn update_ruler(&mut self, content: &Content) {
        let row = content.state.selected().unwrap_or(0);
        match content.targets.len() {
            0 => self.ruler = format!("{}", row + 1),
            _ => {
                self.ruler = format!(
                    "{}/{} {}",
                    match content.targets.iter().enumerate().find(|&(_, a)| a >= &row) {
                        Some((i, _)) => i + 1,
                        None => content.targets.len(),
                    },
                    content.targets.len(),
                    row
                )
            }
        }
    }
}

impl Widget for &Status {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let color = match self.mode {
            Mode::Normal => Color::Blue,
            Mode::Command => Color::Yellow,
        };

        let areas = Layout::horizontal([
            Constraint::Length((self.mode.to_string().len() + 2) as u16),
            Constraint::Fill(1),
            Constraint::Length((self.ruler.len() + 2) as u16),
        ])
        .split(area);

        Paragraph::new(format!(" {} ", self.mode.to_string()))
            .style(Style::new().fg(Color::Black).bg(color))
            .render(areas[0], buf);
        Paragraph::new(format!(" {} ", self.current_dir.to_str().unwrap()))
            .style(Style::new().bg(Color::DarkGray))
            .render(areas[1], buf);
        Paragraph::new(format!(" {} ", self.ruler))
            .style(Style::new().fg(Color::Black).bg(color))
            .render(areas[2], buf);
    }
}

struct Command {
    value: String,
    style: Style,
}

impl Command {
    fn reset(&mut self) {
        self.value = String::from("");
        self.style = Style::new();
    }
}

impl Widget for &Command {
    fn render(self, area: Rect, buf: &mut Buffer) {
        Paragraph::new(self.value.clone())
            .style(self.style)
            .render(area, buf);
    }
}
