// server.rs
use SimpleMessage::commands::{Command, CommandTarget, execute_client_command, parse_command};

use std::collections::HashMap;
use std::io::{BufRead, BufReader, Write};
use std::net::{TcpListener, TcpStream};
use std::sync::{Arc, Mutex};
use std::thread;

struct Client {
    stream: TcpStream,
    addr: std::net::SocketAddr,
    room_id: String,
    room: Arc<Mutex<Vec<TcpStream>>>,
}

// Type alias for the rooms map
type Rooms = Arc<Mutex<HashMap<String, Arc<Mutex<Vec<TcpStream>>>>>>;

// Main Entry Point
fn main() -> std::io::Result<()> {
    let listener = TcpListener::bind("127.0.0.1:7878")?; // Listens for incomming connections
    let rooms: Rooms = Arc::new(Mutex::new(HashMap::new())); // Shared vector of rooms. Each room has a mutex-protected vector of connected clients.
    println!("Listening on 127.0.0.1:7878...");

    // Accept incoming connections and spawn a new thread for each
    for stream in listener.incoming() {
        let stream = stream?;
        let rooms = Arc::clone(&rooms); // Clone the Arc for the thread
        println!("New connection from: {}", stream.peer_addr()?);

        thread::spawn(move || {
            handle_client(stream, rooms);
        });
    }

    Ok(())
}

// Handles an individual client connection
fn handle_client(stream: TcpStream, rooms: Rooms) {
    let mut client: Client = Client {
        stream: stream.try_clone().unwrap(),
        addr: stream.peer_addr().unwrap(),
        room_id: String::new(),
        room: Arc::new(Mutex::new(Vec::new())),
    };

    let mut reader = BufReader::new(stream);
    let mut line = String::new();

    loop {
        line.clear();

        match reader.read_line(&mut line) {
            Ok(0) => break,
            Ok(_) => {
                if line.starts_with('/') {
                    let command: Command = parse_command(&line);
                    execute_server_command(&command, &mut client, &rooms);
                    continue;
                }
                let mut clients = client.room.lock().unwrap();
                let mut i = 0;

                // Broadcast the message to all connected clients
                // Remove clients that fail to send the message
                while i < clients.len() {
                    let client = &mut clients[i];
                    if client.write_all(line.as_bytes()).is_err() {
                        clients.remove(i);
                        continue;
                    }
                    i += 1;
                }
            }
            Err(e) => {
                println!("Client {} disconnected: {}", client.addr, e);
                break;
            }
        }
    }
}

fn execute_server_command(
    command: &Command,
    client: &mut Client,
    rooms: &Arc<Mutex<HashMap<String, Arc<Mutex<Vec<TcpStream>>>>>>,
) {
    match command {
        Command::Join(room_id) => {
            client.room_id = room_id.clone();
            client.room = {
                let mut rooms_guard = rooms.lock().unwrap();

                rooms_guard
                    .entry(room_id.clone())
                    .or_insert_with(|| Arc::new(Mutex::new(Vec::new())))
                    .clone()
            };
            let stream_clone = client.stream.try_clone().unwrap();
            client.room.lock().unwrap().push(stream_clone);
            println!("Client {} joined room {}", client.addr, client.room_id);
        }
        Command::Leave => {}
        _ => {}
    }
}
