// server.rs
use SimpleMessage::commands::{Command, ServerAction, execute_server_command, parse_command};

use std::collections::HashMap;
use std::io::{BufRead, BufReader, Write};
use std::net::{TcpListener, TcpStream};
use std::sync::{Arc, Mutex};
use std::thread;

// Represents a client connected to the server
struct Client {
    stream: TcpStream,
    addr: std::net::SocketAddr,
    username: Option<String>,
    room_id: Option<String>,
    room: Option<Arc<Mutex<Vec<TcpStream>>>>,
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
        username: None,
        room_id: None,
        room: None,
    };

    let mut reader = BufReader::new(stream);
    let mut line = String::new();

    loop {
        line.clear();

        match reader.read_line(&mut line) {
            Ok(0) => break,
            Ok(_) => {
                // Handle commands (e.g., /join, /leave)
                if line.starts_with('/') {
                    handle_commands(&mut client, &rooms, &line);
                    continue;
                }

                // Broadcast the message to all connected clients in the room
                match &client.room {
                    Some(room) => {
                        broadcast_message(room, &line);
                    }
                    None => {
                        let message = "Join a room first with /join <room_id>\n";
                        whisper(&mut client.stream, message);
                    }
                }
            }
            Err(e) => {
                println!("Client {} disconnected: {}", client.addr, e);
                break;
            }
        }
    }
}

// Broadcast a message to all connected clients in a room
fn broadcast_message(room: &Arc<Mutex<Vec<TcpStream>>>, message: &str) {
    let mut clients = room.lock().unwrap();
    let mut i = 0;

    // Broadcast the message to all connected clients
    // Remove clients that fail to send the message
    while i < clients.len() {
        let client = &mut clients[i];
        if client.write_all(message.as_bytes()).is_err() {
            clients.remove(i);
            continue;
        }
        i += 1;
    }
}

// Send a whisper message to a single client
fn whisper(stream: &mut TcpStream, message: &str) {
    let _ = stream.write_all(message.as_bytes());
}

// Handles commands sent by clients
fn handle_commands(
    client: &mut Client,
    rooms: &Arc<Mutex<HashMap<String, Arc<Mutex<Vec<TcpStream>>>>>>,
    line: &str,
) {
    let command: Command = parse_command(line);

    for action in execute_server_command(command) {
        match action {
            // Join: set the client's room_id and room, and add the client's stream to the room
            ServerAction::Join(room_id) => {
                // Using as_deref() to compare room_id with client.room_id as &str (string slice)
                if client.room_id.as_deref() == Some(room_id.as_str()) {
                    continue;
                }
                client.room_id = Some(room_id.clone());
                client.room = Some({
                    let mut rooms_guard = rooms.lock().unwrap();

                    rooms_guard
                        .entry(room_id.clone())
                        .or_insert_with(|| Arc::new(Mutex::new(Vec::new())))
                        .clone()
                });
                let stream_clone = client.stream.try_clone().unwrap();

                if let Some(room) = &client.room {
                    room.lock().unwrap().push(stream_clone);

                    // Broadcast the join message to the room
                    broadcast_message(
                        &room,
                        &format!(
                            "{} has joined the chat\n",
                            client.username.as_deref().unwrap_or("unknown")
                        ),
                    );
                }
                println!("Client {} joined room {}", client.addr, room_id);
            }

            // Disconnect: remove the client from the room (if any) and reset room_id and room to None
            // take() takes the room out of client.room, replacing it with None and returning it
            ServerAction::Disconnect => match client.room.take() {
                Some(room) => {
                    let addr: std::net::SocketAddr = client.addr;

                    // Scope block to limit the lifetime of the lock (prevents deadlock when broadcasting leave message to the room)
                    {
                        let mut clients = room.lock().unwrap();

                        clients.retain(|stream| match stream.peer_addr() {
                            Ok(a) => a != addr,
                            Err(_) => false,
                        });
                    }

                    whisper(
                        &mut client.stream,
                        &format!(
                            "You have left room {}\n",
                            client.room_id.as_deref().unwrap_or("unknown")
                        ),
                    );
                    client.room_id = None;

                    // Broadcast the leave message to the room
                    broadcast_message(
                        &room,
                        &format!(
                            "{} has left the chat\n",
                            client.username.as_deref().unwrap_or("unknown")
                        ),
                    );
                }
                None => {
                    send_client_error(&mut client.stream, "You are not in a room.");
                }
            },

            // SetUsername: set the client's username
            ServerAction::SetUsername(username) => {
                client.username = Some(username);
            }
            _ => {}
        }
    }
}

fn send_client_error(stream: &mut TcpStream, message: &str) {
    let error_msg = format!("[Error] {}\n", message);
    let _ = stream.write_all(error_msg.as_bytes());
}
