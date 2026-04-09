// client.rs
use SimpleMessage::commands::{ClientAction, Command, execute_client_command, parse_command};
use SimpleMessage::ui::{self, ScreenState, UI};
use SimpleMessage::utils::format_message;

use crossterm::event::{Event, KeyCode, KeyEventKind, read};
use std::io::{self, BufRead, BufReader, Write};
use std::net::TcpStream;
use std::sync::mpsc;
use std::thread;

// Main Entry Point
fn main() -> std::io::Result<()> {
    let mut stream = TcpStream::connect("127.0.0.1:7878")?; // Connect to the server
    println!("Connected to server");

    let username: String = username_input()?;

    // Create a channel for sending messages to the UI thread
    let (tx, rx) = mpsc::channel::<String>(); // Multiple Producers, Single Consumer

    let read_stream = stream.try_clone()?; // Clone the stream for reading

    // Spawn a thread to read messages from the server
    thread::spawn(move || {
        read_messages(read_stream, tx);
    });

    // Create the UI
    let mut ui = UI::new();
    let mut input = String::new();
    ui.render(&input, true);

    loop {
        match ui.screen_state {
            ScreenState::Home => {
                ui.render_home(&input);
                if let Ok(Event::Key(event)) = read() {
                    if event.kind != KeyEventKind::Press {
                        continue;
                    }
                    match event.code {
                        KeyCode::Char(c) => {
                            input.push(c);
                        }
                        KeyCode::Backspace => {
                            input.pop();
                        }
                        KeyCode::Enter => {
                            if !input.is_empty() {
                                if input.starts_with('/') {
                                    handle_commands(&mut stream, &mut ui, &mut input)?;
                                }
                                input.clear();
                            }
                        }
                        _ => {}
                    }
                }
            }
            ScreenState::Chat => {
                let mut message_flag = false;
                // Receive any messages from the server and add them to the UI
                while let Ok(message) = rx.try_recv() {
                    ui.add_message(message);
                    message_flag = true;
                }

                // Render the UI and input buffer
                ui.render(&input, message_flag);

                // Read a key event from the user
                if let Ok(Event::Key(event)) = read() {
                    if event.kind != KeyEventKind::Press {
                        continue;
                    }
                    match event.code {
                        KeyCode::Char(c) => {
                            input.push(c);
                        }
                        KeyCode::Backspace => {
                            input.pop();
                        }
                        KeyCode::Enter => {
                            if !input.is_empty() {
                                if input.starts_with('/') {
                                    handle_commands(&mut stream, &mut ui, &mut input)?;
                                } else {
                                    send_line(&mut stream, &format_message(&username, &input))?;
                                }
                                input.clear();
                            }
                        }
                        KeyCode::Up => {
                            ui.increase_start_offset();
                            ui.render(&input, true);
                        }
                        KeyCode::Down => {
                            ui.decrease_start_offset();
                            ui.render(&input, true);
                        }
                        KeyCode::Esc => break,
                        _ => {}
                    }
                }
            }
        }
    }

    Ok(())
}

// Send a string to the server
fn send_line(stream: &mut TcpStream, line: &str) -> Result<(), io::Error> {
    stream.write_all(line.as_bytes())?;
    stream.write_all(b"\n")?;
    Ok(())
}

// Prompt the user for their username and return it
fn username_input() -> Result<String, io::Error> {
    let mut input = String::new();

    print!("Enter your username: ");
    io::stdout().flush()?;
    std::io::stdin().read_line(&mut input).ok();

    Ok(input.trim().to_string())
}

// Handle commands entered by the user (prefixed with '/')
fn handle_commands(
    stream: &mut TcpStream,
    ui: &mut UI,
    input: &mut String,
) -> Result<(), io::Error> {
    let command: Command = parse_command(&input); // Parse the command entered by the user

    // Execute the command and handle the actions
    for action in execute_client_command(command) {
        match action {
            ClientAction::Quit => {
                std::process::exit(0);
            }
            ClientAction::ChangeScreen(state) => {
                ui.screen_state = state;
                input.clear();
            }
            ClientAction::Forward() => {
                send_line(stream, &input)?;
                ui.screen_state = ScreenState::Chat;
                input.clear();
            }
        }
    }

    Ok(())
}

// Read messages from the server and print them
fn read_messages(read_stream: TcpStream, tx: mpsc::Sender<String>) {
    let mut reader = BufReader::new(read_stream);
    let mut line = String::new();

    loop {
        line.clear();
        match reader.read_line(&mut line) {
            Ok(0) => break,
            Ok(_) => {
                let _ = tx.send(line.trim().to_string()); // Send the line to the UI thread
            }
            Err(e) => {
                println!("Error: {}", e);
                break;
            }
        }
    }
}
