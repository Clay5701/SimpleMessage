use crossterm::{
    cursor::MoveTo,
    execute,
    style::Print,
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
    pub fn render(&self, input: &str, new_message: bool) {
        let mut stdout = stdout();
        let (cols, rows) = size().unwrap();
        let max_messages = rows as usize - 1;

        if new_message {
            // Clear the screen
            execute!(stdout, Clear(ClearType::All)).unwrap();

            // Determine which messages to display
            let start = self.messages.len().saturating_sub(max_messages);

            // Draw messages
            for (i, message) in self.messages[start..].iter().enumerate() {
                execute!(stdout, MoveTo(0, i as u16)).unwrap();
                write!(stdout, "{}\r\n", message).unwrap();
            }

            stdout.flush().unwrap();
        }

        let prompt = format!("> {}", input);

        // Draw prompt and input all in one go to avoid cursor position issues and prevent flickering
        execute!(
            stdout,
            MoveTo(0, rows as u16 - 1),
            Print(format!("{:<width$}", prompt, width = cols as usize - 1)),
            MoveTo(prompt.len() as u16, rows as u16 - 1)
        )
        .unwrap();
        stdout.flush().unwrap();
    }
}
