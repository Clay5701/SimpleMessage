// client.rs
use SimpleMessage::commands::{Command, CommandTarget, execute_client_command, parse_command};
use SimpleMessage::ui::{ScreenState, UI};
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

    // Prompt the user for the room ID and send it to the server
    // let room_id: String = room_input()?;
    // send_line(&mut stream, &room_id)?;

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

    // Create variable to store the current screen state
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
                                    let command: Command = parse_command(&input);
                                    match execute_client_command(command, &mut ui) {
                                        CommandTarget::ClientHandled => {
                                            input.clear();
                                        }
                                        CommandTarget::ServerHandled => {
                                            send_line(&mut stream, &input)?;
                                            ui.screen_state = ScreenState::Chat;
                                            input.clear();
                                        }
                                    }
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
                                    send_line(&mut stream, &input)?;
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

// Prompt the user for the room ID and return it
fn room_input() -> Result<String, io::Error> {
    loop {
        let mut input = String::new();

        print!("Enter a six-digit room ID: ");
        io::stdout().flush()?;
        std::io::stdin().read_line(&mut input).ok();

        let room_id = input.trim();

        // Validate the room ID (must be 6 digits)
        if room_id.len() == 6 && room_id.chars().all(|c| c.is_ascii_digit()) {
            return Ok(room_id.to_string());
        } else {
            println!("Invalid room ID. Please enter a six-digit number.");
        }
    }
}

// Prompt the user for their username and return it
fn username_input() -> Result<String, io::Error> {
    let mut input = String::new();

    print!("Enter your username: ");
    io::stdout().flush()?;
    std::io::stdin().read_line(&mut input).ok();

    Ok(input.trim().to_string())
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
