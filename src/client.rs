mod utils;
use utils::format_message;

use std::io::{self, BufRead, BufReader, Write};
use std::net::TcpStream;
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

    let read_stream = stream.try_clone()?; // Clone the stream for reading

    // Spawn a thread to read messages from the server
    thread::spawn(move || {
        read_messages(read_stream);
    });

    // Read messages from the user and send them to the server
    let stdin = io::stdin();
    for line in stdin.lock().lines() {
        let line = line?;
        let formatted_line = format_message(&username, &line);
        stream.write_all(formatted_line.as_bytes())?;
        stream.write_all(b"\n")?;
    }

    Ok(())
}

// Read messages from the server and print them
fn read_messages(read_stream: TcpStream) {
    let mut reader = BufReader::new(read_stream);
    let mut line = String::new();

    loop {
        line.clear();
        match reader.read_line(&mut line) {
            Ok(0) => break,
            Ok(_) => println!("{}", line.trim()),
            Err(e) => {
                println!("Error: {}", e);
                break;
            }
        }
    }
}
