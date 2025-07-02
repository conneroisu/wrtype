//! Advanced key sequences example
//!
//! This example demonstrates complex key sequences, timing control, and
//! sophisticated automation scenarios using wrtype's Command API.
//! It shows how to build complex workflows that simulate real user interactions.
//!
//! Run with: cargo run --example advanced_sequences

use wrtype::{WrtypeClient, Command, Modifier};
use std::time::Duration;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("Creating wrtype client...");
    
    let mut client = WrtypeClient::new()?;
    
    println!("Demonstrating advanced key sequences...");
    
    // Sequence 1: Simulating a text editor workflow
    println!("1. Text editor workflow simulation...");
    
    let editor_workflow = vec![
        // Open a new file (Ctrl+N)
        Command::ModPress(Modifier::Ctrl),
        Command::KeyPress("n".to_string()),
        Command::KeyRelease("n".to_string()),
        Command::ModRelease(Modifier::Ctrl),
        Command::Sleep(Duration::from_millis(500)),
        
        // Type some content
        Command::Text {
            text: "// This is a sample file\nfn main() {\n    println!(\"Hello, World!\");\n}".to_string(),
            delay: Duration::from_millis(50),
        },
        Command::Sleep(Duration::from_millis(300)),
        
        // Select the println! line using Shift+arrow keys
        Command::KeyPress("Up".to_string()),       // Move up one line
        Command::KeyRelease("Up".to_string()),
        Command::KeyPress("End".to_string()),       // Move to end of line
        Command::KeyRelease("End".to_string()),
        Command::ModPress(Modifier::Shift),        // Start selection
        Command::KeyPress("Home".to_string()),      // Select to beginning of line
        Command::KeyRelease("Home".to_string()),
        Command::ModRelease(Modifier::Shift),
        
        // Copy the selected text (Ctrl+C)
        Command::ModPress(Modifier::Ctrl),
        Command::KeyPress("c".to_string()),
        Command::KeyRelease("c".to_string()),
        Command::ModRelease(Modifier::Ctrl),
        
        // Move to end of file and paste
        Command::ModPress(Modifier::Ctrl),
        Command::KeyPress("End".to_string()),
        Command::KeyRelease("End".to_string()),
        Command::ModRelease(Modifier::Ctrl),
        Command::KeyPress("Return".to_string()),
        Command::KeyRelease("Return".to_string()),
        Command::ModPress(Modifier::Ctrl),
        Command::KeyPress("v".to_string()),
        Command::KeyRelease("v".to_string()),
        Command::ModRelease(Modifier::Ctrl),
    ];
    
    client.execute_commands(editor_workflow)?;
    client.sleep(Duration::from_millis(1000))?;
    
    // Sequence 2: Simulating form filling
    println!("2. Form filling simulation...");
    
    let form_filling = vec![
        // Clear existing content
        Command::ModPress(Modifier::Ctrl),
        Command::KeyPress("a".to_string()),
        Command::KeyRelease("a".to_string()),
        Command::ModRelease(Modifier::Ctrl),
        Command::KeyPress("Delete".to_string()),
        Command::KeyRelease("Delete".to_string()),
        
        // Fill out a form with tab navigation
        Command::Text {
            text: "John Doe".to_string(),
            delay: Duration::from_millis(80),
        },
        Command::KeyPress("Tab".to_string()),       // Move to next field
        Command::KeyRelease("Tab".to_string()),
        Command::Sleep(Duration::from_millis(200)),
        
        Command::Text {
            text: "john.doe@example.com".to_string(),
            delay: Duration::from_millis(60),
        },
        Command::KeyPress("Tab".to_string()),
        Command::KeyRelease("Tab".to_string()),
        Command::Sleep(Duration::from_millis(200)),
        
        Command::Text {
            text: "+1-555-123-4567".to_string(),
            delay: Duration::from_millis(100),
        },
        Command::KeyPress("Tab".to_string()),
        Command::KeyRelease("Tab".to_string()),
        
        // Fill in address with line breaks
        Command::Text {
            text: "123 Main Street".to_string(),
            delay: Duration::from_millis(70),
        },
        Command::KeyPress("Return".to_string()),
        Command::KeyRelease("Return".to_string()),
        Command::Text {
            text: "Anytown, ST 12345".to_string(),
            delay: Duration::from_millis(70),
        },
    ];
    
    client.execute_commands(form_filling)?;
    client.sleep(Duration::from_millis(1000))?;
    
    // Sequence 3: Complex modifier combinations
    println!("3. Complex modifier combinations...");
    
    let complex_modifiers = vec![
        // Clear and start fresh
        Command::ModPress(Modifier::Ctrl),
        Command::KeyPress("a".to_string()),
        Command::KeyRelease("a".to_string()),
        Command::ModRelease(Modifier::Ctrl),
        Command::KeyPress("Delete".to_string()),
        Command::KeyRelease("Delete".to_string()),
        
        // Type some text to work with
        Command::Text {
            text: "Testing complex key combinations".to_string(),
            delay: Duration::from_millis(30),
        },
        
        // Ctrl+Shift+Left to select word by word
        Command::ModPress(Modifier::Ctrl),
        Command::ModPress(Modifier::Shift),
        Command::KeyPress("Left".to_string()),
        Command::KeyRelease("Left".to_string()),
        Command::KeyPress("Left".to_string()),
        Command::KeyRelease("Left".to_string()),
        Command::ModRelease(Modifier::Shift),
        Command::ModRelease(Modifier::Ctrl),
        
        // Make selected text uppercase (this would depend on the application)
        Command::Text {
            text: "COMBINATIONS".to_string(),
            delay: Duration::from_millis(50),
        },
        
        Command::Sleep(Duration::from_millis(500)),
        
        // Demonstrate Alt+F4 (close window - be careful!)
        // Comment this out if you don't want to close windows
        /*
        Command::ModPress(Modifier::Alt),
        Command::KeyPress("F4".to_string()),
        Command::KeyRelease("F4".to_string()),
        Command::ModRelease(Modifier::Alt),
        */
    ];
    
    client.execute_commands(complex_modifiers)?;
    
    // Sequence 4: Gaming-style key sequences (WASD movement simulation)
    println!("4. Gaming-style key sequences...");
    
    let gaming_sequence = vec![
        Command::Text {
            text: "\n\nSimulating game controls:\n".to_string(),
            delay: Duration::from_millis(30),
        },
        
        // Move forward (W)
        Command::KeyPress("w".to_string()),
        Command::Sleep(Duration::from_millis(100)),
        Command::KeyRelease("w".to_string()),
        Command::Text { text: "Forward ".to_string(), delay: Duration::from_millis(0) },
        
        // Strafe left (A)
        Command::KeyPress("a".to_string()),
        Command::Sleep(Duration::from_millis(100)),
        Command::KeyRelease("a".to_string()),
        Command::Text { text: "Left ".to_string(), delay: Duration::from_millis(0) },
        
        // Move backward (S)
        Command::KeyPress("s".to_string()),
        Command::Sleep(Duration::from_millis(100)),
        Command::KeyRelease("s".to_string()),
        Command::Text { text: "Back ".to_string(), delay: Duration::from_millis(0) },
        
        // Strafe right (D)
        Command::KeyPress("d".to_string()),
        Command::Sleep(Duration::from_millis(100)),
        Command::KeyRelease("d".to_string()),
        Command::Text { text: "Right ".to_string(), delay: Duration::from_millis(0) },
        
        // Jump (Space)
        Command::KeyPress("space".to_string()),
        Command::Sleep(Duration::from_millis(50)),
        Command::KeyRelease("space".to_string()),
        Command::Text { text: "Jump!".to_string(), delay: Duration::from_millis(0) },
    ];
    
    client.execute_commands(gaming_sequence)?;
    
    println!("Advanced sequences demonstration complete!");
    Ok(())
}