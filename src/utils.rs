// utils.rs
pub fn format_message(username: &str, message: &str) -> String {
    format!("[{}]: {}", username, message)
}
