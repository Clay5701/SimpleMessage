use crossterm::{
    cursor::MoveTo,
    execute,
    terminal::{Clear, ClearType, size},
};
use std::io::{Write, stdout};

// UI for displaying messages and input
pub struct UI {
    pub messages: Vec<String>,
}

impl UI {
    // Create a new UI instance
    pub fn new() -> Self {
        Self {
            messages: Vec::new(),
        }
    }

    // Add a message to the UI
    pub fn add_message(&mut self, message: String) {
        self.messages.push(message);
    }

    // Render the UI, displaying messages and input
    pub fn render(&self, input: &str) {
        let mut stdout = stdout();

        let (_, rows) = size().unwrap();
        let max_messages = rows as usize - 1;

        // Clear the screen
        execute!(stdout, Clear(ClearType::All)).unwrap();

        // Determine which messages to display
        let start = self.messages.len().saturating_sub(max_messages);

        // Draw messages
        for (i, message) in self.messages[start..].iter().enumerate() {
            execute!(stdout, MoveTo(0, i as u16)).unwrap();
            write!(stdout, "{}\r\n", message).unwrap();
        }

        // Move cursor to the bottom
        execute!(stdout, MoveTo(0, rows as u16 - 1)).unwrap();
        write!(stdout, "> {}", input).unwrap();

        stdout.flush().unwrap();
    }
}
