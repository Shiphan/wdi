use std::env;

use std::fs;

/*
use std::io::{self, stdout};

use ratatui::{
    crossterm::{
        event::{self, Event, KeyCode},
        terminal::{
            disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen,
        },
        ExecutableCommand,
    },
    prelude::*, widgets::*,
};
*/

fn main() {
    let wd = env::current_dir().unwrap();
    let list = fs::read_dir(wd.clone()).unwrap();

    println!("{}", wd.display());
    println!("-----");
    for item in list {
        println!("{:?}", item.unwrap().file_name())
    }
}
