// client.rs
mod ui;
use ui::UI;

mod utils;
use utils::format_message;

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

    // Create the UI and input buffer
    let mut ui = UI::new();
    let mut input = String::new();
    ui.render(&input, true);
    loop {
        let mut message_flag = false;
        // Receive any messages from the server and add them to the UI
        while let Ok(message) = rx.try_recv() {
            ui.add_message(message);
            message_flag = true;
        }

        // Render the UI and input buffer
        ui.render(&input, message_flag);

        // Read a key event from the user
        if let Event::Key(event) = read().unwrap() {
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
                        let formatted_line = format_message(&username, &input);
                        stream.write_all(formatted_line.as_bytes())?;
                        stream.write_all(b"\n")?;
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
