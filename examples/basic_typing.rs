//! Basic text typing example
//!
//! This example demonstrates the simplest use case for wrtype: typing text.
//! It shows how to create a client, type some text, and handle errors.
//!
//! Run with: cargo run --example basic_typing

use wrtype::WrtypeClient;
use std::time::Duration;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("Creating wrtype client...");
    
    // Create a new wrtype client and connect to Wayland
    let mut client = WrtypeClient::new()?;
    
    println!("Typing basic text...");
    
    // Type some simple text
    client.type_text("Hello, World!")?;
    
    // Wait a moment
    client.sleep(Duration::from_millis(500))?;
    
    // Type text with Unicode characters
    client.type_text(" 🦀 Rust on Wayland! 🚀")?;
    
    // Wait a moment
    client.sleep(Duration::from_millis(500))?;
    
    // Type with a delay between characters (slow typing effect)
    println!("Typing with character delay...");
    client.type_text_with_delay(
        "\nThis text is typed slowly...",
        Duration::from_millis(100)
    )?;
    
    println!("Done!");
    Ok(())
}