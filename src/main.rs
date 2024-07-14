use std::env;
use std::fs;

use std::io::{self, stdout, Result};

use ratatui::{
    crossterm::{
        event::{self, Event, KeyCode},
        terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
        ExecutableCommand,
    },
    prelude::*,
    widgets::*,
};

fn main() -> Result<()> {
    enable_raw_mode()?;
    stdout().execute(EnterAlternateScreen)?;

    let mut terminal = Terminal::new(CrosstermBackend::new(stdout()))?;

    let mut should_quit = false;
    while !should_quit {
        terminal.draw(ui)?;
        should_quit = handle_events()?;
    }

    disable_raw_mode()?;
    stdout().execute(LeaveAlternateScreen)?;

    Ok(())
}

fn handle_events() -> Result<bool> {
    if event::poll(std::time::Duration::from_millis(50))? {
        if let Event::Key(key) = event::read()? {
            if key.kind == event::KeyEventKind::Press && key.code == KeyCode::Char('q') {
                return Ok(true);
            }
        }
    }
    Ok(false)
}

fn ui(frame: &mut Frame) {
    let areas = Layout::new(
        Direction::Vertical,
        [
            Constraint::Length(1),
            Constraint::Fill(1),
            Constraint::Length(1),
        ],
    )
    .split(frame.size());

    let current_dir = env::current_dir().unwrap();
    let read_dir = fs::read_dir(current_dir.clone()).unwrap();
    let items: Vec<String> = read_dir
        .map(|item| item.unwrap().file_name().to_str().unwrap().to_owned())
        .collect();

    let list = List::new(items)
        // .block(Block::bordered().title(current_dir.to_str().unwrap()))
        .style(Style::default().fg(Color::White))
        .highlight_style(Style::default().add_modifier(Modifier::ITALIC))
        .highlight_symbol(">>")
        .repeat_highlight_symbol(true)
        .direction(ListDirection::TopToBottom);

    frame.render_widget(Paragraph::new(current_dir.to_str().unwrap()), areas[0]);
    frame.render_widget(list, areas[1]);
    frame.render_widget(Paragraph::new("end line here~~~"), areas[2]);
}
