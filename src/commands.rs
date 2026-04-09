// commands.rs
use crate::ui::{ScreenState, UI};

use std::io;
use std::net::TcpStream;

pub enum CommandTarget {
    ClientHandled,
    ServerHandled,
}

pub enum Command {
    Join(String),
    Leave,
    ListRooms,
    ListUsers,
    Quit,
    Help,
    Unknown(String),
}

pub fn parse_command(input: &str) -> Command {
    let parts: Vec<&str> = input.trim().split_whitespace().collect();
    match parts.as_slice() {
        ["/join", room] => {
            if valid_room(&room) {
                Command::Join(room.to_string())
            } else {
                Command::Unknown(input.to_string())
            }
        }
        ["/leave"] => Command::Leave,
        ["/rooms"] => Command::ListRooms,
        ["/users"] => Command::ListUsers,
        ["/quit"] => Command::Quit,
        ["/help"] => Command::Help,
        _ => Command::Unknown(input.to_string()),
    }
}

pub fn execute_client_command(command: Command, ui: &mut UI) -> CommandTarget {
    match command {
        Command::Quit => {
            std::process::exit(0);
        }
        Command::Help => {
            ui.screen_state = ScreenState::Home;
            CommandTarget::ClientHandled
        }
        Command::Unknown(_) => CommandTarget::ServerHandled,
        _ => CommandTarget::ServerHandled,
    }
}

fn valid_room(room: &str) -> bool {
    room.len() == 6 && room.chars().all(|c| c.is_ascii_digit())
}
