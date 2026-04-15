// commands.rs
use crate::ui::ScreenState;

// Represents an action to be taken by the client
pub enum ClientAction {
    ChangeScreen(ScreenState),
    Quit,
    SetUsername(String),
    Forward,
}

// Represents an action to be taken by the server
pub enum ServerAction {
    Alert(String),
    SetUsername(String),
    Join(String),
    Disconnect,
}

// Represents possible commands that can be executed by the client or server
pub enum Command {
    Join(String),
    Leave,
    ListRooms,
    ListUsers,
    SetUsername(String),
    Quit,
    Help,
    Unknown(String),
}

// Parses a raw input string into a Command enum variant
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
        ["/set_username", username] => Command::SetUsername(username.to_string()),
        ["/quit"] => Command::Quit,
        ["/help"] => Command::Help,
        _ => Command::Unknown(input.to_string()),
    }
}

// Executes a client command and returns a vector of ClientAction variants
pub fn execute_client_command(command: Command) -> Vec<ClientAction> {
    match command {
        Command::Join(_) => {
            vec![
                ClientAction::ChangeScreen(ScreenState::Chat),
                ClientAction::Forward,
            ]
        }
        Command::Quit => {
            vec![ClientAction::Quit]
        }
        Command::Help => {
            vec![ClientAction::ChangeScreen(ScreenState::Home)]
        }
        Command::SetUsername(username) => {
            vec![ClientAction::SetUsername(username), ClientAction::Forward]
        }
        _ => vec![ClientAction::Forward],
    }
}

// Executes a server command and returns a vector of ServerAction variants
pub fn execute_server_command(command: Command) -> Vec<ServerAction> {
    match command {
        Command::Join(room) => {
            let alert: String = format!("You have successfully joined room {}!", room);
            vec![ServerAction::Join(room), ServerAction::Alert(alert)]
        }
        Command::Leave => {
            let alert: String = format!("You have successfully left the room.");
            vec![ServerAction::Disconnect, ServerAction::Alert(alert)]
        }
        Command::SetUsername(username) => {
            vec![ServerAction::SetUsername(username)]
        }
        _ => vec![],
    }
}

// Validates a room string to ensure it is a 6-digit number
fn valid_room(room: &str) -> bool {
    room.len() == 6 && room.chars().all(|c| c.is_ascii_digit())
}
