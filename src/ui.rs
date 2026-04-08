// ui.rs
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
    pub start_offset: usize,
    pub rows: u16,
    pub cols: u16,
    pub max_messages: usize,
}

impl UI {
    // Create a new UI instance
    pub fn new() -> Self {
        let (cols, rows) = size().unwrap();

        Self {
            messages: Vec::new(),
            start_offset: 0,
            rows,
            cols,
            max_messages: rows as usize - 1,
        }
    }

    // Add a message to the UI
    pub fn add_message(&mut self, message: String) {
        self.messages.push(message);

        // Increase start offset to "freeze" the messages while scrolling
        if self.start_offset > 0 {
            self.increase_start_offset();
        }
    }

    // Render the UI, displaying messages and input
    pub fn render(&mut self, input: &str, new_message: bool) {
        self.update_dimensions(); // Update dimensions before rendering
        let mut stdout = stdout(); // Get a handle to the terminal stdout

        if new_message {
            // Clear the screen
            execute!(stdout, Clear(ClearType::All)).unwrap();

            // Determine which messages to display
            let start = self
                .messages
                .len()
                .saturating_sub(self.max_messages)
                .saturating_sub(self.start_offset); // Clamp start to avoid negative start values

            // Draw messages
            for (i, message) in self.messages[start..]
                .iter()
                .take(self.max_messages) // Limit to max_messages to avoid index out of bounds
                .enumerate()
            // Enumerate to get the index for MoveTo
            {
                execute!(stdout, MoveTo(0, i as u16)).unwrap();
                write!(stdout, "{}", message).unwrap();
            }

            stdout.flush().unwrap();
        }

        let prompt = format!("> {}", input);

        // Draw prompt and input all in one go to avoid cursor position issues and prevent flickering
        // Potential bug: If prompt/input exceeds cols, it will be truncated without warning
        // Potential bug: Inputing a character that has a non-standard width (emojis) will break alignment due to using prompt.len() to calculate cursor position
        execute!(
            stdout,
            MoveTo(0, self.rows - 1),
            Print(format!(
                "{:<width$}",
                prompt,
                width = self.cols as usize - 1
            )),
            MoveTo(prompt.len() as u16, self.rows - 1)
        )
        .unwrap();
        stdout.flush().unwrap();
    }

    // Update terminal dimensions (cols, rows, max_messages)
    fn update_dimensions(&mut self) {
        let (cols, rows) = size().unwrap();
        self.cols = cols;
        self.rows = rows;
        self.max_messages = rows as usize - 1;
    }

    // Increase start offset, clamping to avoid index out of bounds
    pub fn increase_start_offset(&mut self) {
        self.start_offset += 1;
        self.start_offset = self
            .start_offset
            .min(self.messages.len().saturating_sub(self.max_messages));
    }

    // Decrease start offset, clamping to avoid index out of bounds
    pub fn decrease_start_offset(&mut self) {
        self.start_offset = self.start_offset.saturating_sub(1);
    }
}
