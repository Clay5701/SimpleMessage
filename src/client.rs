mod ui;
use ui::UI;

mod utils;
use utils::format_message;

use crossterm::event::{Event, KeyCode, read};
use crossterm::terminal::{disable_raw_mode, enable_raw_mode};

use std::io::{self, BufRead, BufReader, Write};
use std::net::TcpStream;
use std::sync::mpsc;
use std::thread;

// Main Entry Point
fn main() -> std::io::Result<()> {
    let mut stream = TcpStream::connect("127.0.0.1:7878")?; // Connect to the server
    println!("Connected to server");

    let mut username = String::new();
    print!("Enter your username: ");
    io::stdout().flush()?;
    io::stdin().read_line(&mut username)?;
    let username = username.trim().to_string();

    // Enable raw mode for the terminal (Fixes double key input issue)
    enable_raw_mode()?;

    // Create a channel for sending messages to the UI thread
    let (tx, rx) = mpsc::channel::<String>(); // Multiple Producer, Single Consumer

    let read_stream = stream.try_clone()?; // Clone the stream for reading

    // Spawn a thread to read messages from the server
    thread::spawn(move || {
        read_messages(read_stream, tx);
    });

    // Create the UI and input buffer
    let mut ui = UI::new();
    let mut input = String::new();

    loop {
        // Receive any messages from the server and add them to the UI
        while let Ok(message) = rx.try_recv() {
            ui.add_message(message);
        }

        // Render the UI and input buffer
        ui.render(&input);

        // Read a key event from the user
        if let Event::Key(event) = read().unwrap() {
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

                        ui.add_message(formatted_line);

                        input.clear();
                    }
                }
                KeyCode::Esc => break,
                _ => {}
            }
        }
    }

    disable_raw_mode()?;

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
                let _ = tx.send(line.trim().to_string());
            }
            Err(e) => {
                println!("Error: {}", e);
                break;
            }
        }
    }
}
