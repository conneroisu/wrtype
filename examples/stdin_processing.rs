//! Stdin processing example
//!
//! This example demonstrates how to read text from stdin and type it using wrtype.
//! This is useful for integrating wrtype into shell pipelines or processing
//! large amounts of text from files.
//!
//! Run with: 
//!   echo "Hello from stdin!" | cargo run --example stdin_processing
//!   cargo run --example stdin_processing < some_file.txt

use wrtype::{WrtypeClient, Command};
use std::io::{self, Read};
use std::time::Duration;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("Creating wrtype client...");
    
    let mut client = WrtypeClient::new()?;
    
    // Check if there's data available on stdin
    let mut stdin_data = String::new();
    let stdin_result = io::stdin().read_to_string(&mut stdin_data);
    
    match stdin_result {
        Ok(0) => {
            // No data on stdin, demonstrate with sample text
            println!("No stdin data provided. Demonstrating with sample text...");
            println!("Try running: echo 'Hello from pipe!' | cargo run --example stdin_processing");
            
            // Simulate reading from stdin by typing sample text
            let sample_text = "This text simulates stdin input.\nIt can contain multiple lines!\nAnd Unicode: 🌟✨🔥";
            
            println!("Typing sample stdin text...");
            client.type_text_with_delay(sample_text, Duration::from_millis(50))?;
        }
        Ok(_) => {
            // We have stdin data, process it
            println!("Processing {} bytes from stdin...", stdin_data.len());
            
            // Option 1: Type all at once
            println!("Typing stdin data (fast)...");
            client.type_text(&stdin_data)?;
            
            client.sleep(Duration::from_millis(1000))?;
            
            // Option 2: Type with character delay for effect
            println!("Typing stdin data again (with delay)...");
            client.type_text_with_delay(&stdin_data, Duration::from_millis(20))?;
        }
        Err(e) => {
            eprintln!("Error reading from stdin: {}", e);
            return Err(e.into());
        }
    }
    
    // Demonstrate using the Command API for stdin processing
    println!("\nDemonstrating Command API for stdin processing...");
    
    // Create a command that would read from stdin (if available)
    let _stdin_command = Command::StdinText {
        delay: Duration::from_millis(30)
    };
    
    // Note: This would normally read from stdin, but since we've already read it,
    // we'll demonstrate with a different approach
    client.type_text("\n--- End of stdin processing example ---")?;
    
    println!("Stdin processing complete!");
    Ok(())
}