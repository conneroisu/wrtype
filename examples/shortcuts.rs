//! Keyboard shortcuts and hotkeys example
//!
//! This example demonstrates how to send common keyboard shortcuts and
//! complex key combinations using wrtype. It shows both the high-level
//! shortcut API and manual modifier handling.
//!
//! Run with: cargo run --example shortcuts

use wrtype::{WrtypeClient, Modifier};
use std::time::Duration;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("Creating wrtype client...");
    
    let mut client = WrtypeClient::new()?;
    
    println!("Demonstrating common keyboard shortcuts...");
    
    // Type some text first to have something to work with
    client.type_text("This is some sample text for demonstration.")?;
    client.sleep(Duration::from_millis(500))?;
    
    // Select all text (Ctrl+A)
    println!("Sending Ctrl+A (Select All)...");
    client.send_shortcut(&[Modifier::Ctrl], "a")?;
    client.sleep(Duration::from_millis(500))?;
    
    // Copy to clipboard (Ctrl+C)
    println!("Sending Ctrl+C (Copy)...");
    client.send_shortcut(&[Modifier::Ctrl], "c")?;
    client.sleep(Duration::from_millis(500))?;
    
    // Move to end of line (End key)
    println!("Sending End key...");
    client.type_key("End")?;
    client.sleep(Duration::from_millis(500))?;
    
    // Add a new line
    client.type_key("Return")?;
    
    // Paste from clipboard (Ctrl+V)
    println!("Sending Ctrl+V (Paste)...");
    client.send_shortcut(&[Modifier::Ctrl], "v")?;
    client.sleep(Duration::from_millis(500))?;
    
    // Demonstrate more complex shortcuts
    
    // Open new tab (Ctrl+Shift+T)
    println!("Sending Ctrl+Shift+T (New Tab)...");
    client.send_shortcut(&[Modifier::Ctrl, Modifier::Shift], "t")?;
    client.sleep(Duration::from_millis(500))?;
    
    // Demonstrate Alt+Tab (switch windows)
    println!("Sending Alt+Tab (Switch Windows)...");
    client.send_shortcut(&[Modifier::Alt], "Tab")?;
    client.sleep(Duration::from_millis(500))?;
    
    // Demonstrate manual modifier control for complex sequences
    println!("Demonstrating manual modifier control...");
    
    // Hold Shift and type multiple keys (creates uppercase letters)
    client.press_modifier(Modifier::Shift)?;
    client.type_text("THESE LETTERS ARE UPPERCASE")?;
    client.release_modifier(Modifier::Shift)?;
    
    client.sleep(Duration::from_millis(500))?;
    
    // Demonstrate function keys
    println!("Sending function keys...");
    client.type_key("F1")?;
    client.sleep(Duration::from_millis(200))?;
    client.type_key("F12")?;
    client.sleep(Duration::from_millis(200))?;
    
    // Demonstrate arrow keys
    println!("Sending arrow keys...");
    client.type_key("Left")?;
    client.sleep(Duration::from_millis(200))?;
    client.type_key("Right")?;
    client.sleep(Duration::from_millis(200))?;
    client.type_key("Up")?;
    client.sleep(Duration::from_millis(200))?;
    client.type_key("Down")?;
    
    // Demonstrate Page Up/Down
    println!("Sending Page Up/Down...");
    client.type_key("Page_Up")?;
    client.sleep(Duration::from_millis(200))?;
    client.type_key("Page_Down")?;
    
    // Demonstrate Escape key
    println!("Sending Escape key...");
    client.type_key("Escape")?;
    
    println!("Shortcut demonstration complete!");
    Ok(())
}