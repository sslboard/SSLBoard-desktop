/// A simple greeting command for testing the Tauri-Rust bridge.
/// This command demonstrates basic string processing and command invocation.
///
/// # Arguments
/// * `name` - The name to greet
///
/// # Returns
/// A greeting string from Rust
#[tauri::command]
pub fn greet(name: &str) -> String {
    format!("Hello, {}! You've been greeted from Rust!", name)
}
