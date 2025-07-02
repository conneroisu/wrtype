// Main entry point for wrtype - a Rust implementation of wtype (xdotool type for Wayland)
//
// This module handles command-line argument parsing, command sequencing, and orchestrates
// the interaction between the Wayland virtual keyboard protocol and the XKB keymap system.

mod executor;
mod keymap;
mod wayland;

use clap::Parser;
use executor::CommandExecutor;
use std::time::Duration;

/// Command-line arguments structure using clap for automatic parsing and help generation.
/// This structure mirrors the original wtype interface for full compatibility.
#[derive(Parser)]
#[command(name = "wrtype")]
#[command(about = "xdotool type for Wayland")]
#[command(version)]
pub struct Args {
    /// Text to type (use -- before text to avoid parsing as options)
    /// Examples: wrtype "hello world", wrtype -- "-special-text"
    pub text: Vec<String>,
    
    /// Press modifier (shift, capslock, ctrl, logo, win, alt, altgr)
    /// Can be used multiple times: -M shift -M ctrl
    #[arg(short = 'M', value_name = "MOD")]
    pub press_mod: Vec<String>,
    
    /// Release modifier (shift, capslock, ctrl, logo, win, alt, altgr)
    /// Should be paired with corresponding -M commands: -M ctrl ... -m ctrl
    #[arg(short = 'm', value_name = "MOD")]
    pub release_mod: Vec<String>,
    
    /// Press key (using XKB key names like "Return", "Left", "space")
    /// Key remains pressed until explicitly released with -p
    #[arg(short = 'P', value_name = "KEY")]
    pub press_key: Vec<String>,
    
    /// Release key (using XKB key names)
    /// Should be paired with corresponding -P commands
    #[arg(short = 'p', value_name = "KEY")]
    pub release_key: Vec<String>,
    
    /// Type (press and release) key in one action
    /// Equivalent to -P <key> -p <key>
    #[arg(short = 'k', value_name = "KEY")]
    pub type_key: Vec<String>,
    
    /// Sleep for TIME milliseconds between keystrokes (default: 0)
    /// Applies to all subsequent text typing commands
    #[arg(short = 'd', value_name = "TIME", default_value = "0")]
    pub delay: u64,
    
    /// Sleep for TIME milliseconds before interpreting following options
    /// Used for timing complex key sequences: -P key -s 1000 -p key
    #[arg(short = 's', value_name = "TIME")]
    pub sleep: Vec<u64>,
    
    /// Read text from stdin instead of command line arguments
    /// Useful for piping: echo "text" | wrtype --stdin
    #[arg(long)]
    pub stdin: bool,
}

/// Internal command representation after parsing command-line arguments.
/// Each command represents a single action to be executed in sequence.
#[derive(Debug, Clone)]
pub enum Command {
    /// Type a string of text with specified delay between characters
    Text { text: String, delay: Duration },
    /// Press a modifier key (adds to current modifier state)
    ModPress(Modifier),
    /// Release a modifier key (removes from current modifier state)
    ModRelease(Modifier),
    /// Press a named key (key stays pressed until released)
    KeyPress(String),
    /// Release a named key
    KeyRelease(String),
    /// Sleep for specified duration (for timing control)
    Sleep(Duration),
    /// Read and type text from stdin with specified delay
    StdinText { delay: Duration },
}

/// Modifier keys with their corresponding bit values for Wayland protocol.
/// These values match the modifier mask constants used in XKB and Wayland.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Modifier {
    /// Shift key - bit 0 (value 1)
    Shift = 1,
    /// Caps Lock - bit 1 (value 2) - handled as locked modifier
    CapsLock = 2,
    /// Control key - bit 2 (value 4)
    Ctrl = 4,
    /// Alt key - bit 3 (value 8)
    Alt = 8,
    /// Logo/Super/Windows key - bit 6 (value 64)
    Logo = 64,
    /// AltGr (right Alt) key - bit 7 (value 128)
    AltGr = 128,
}

impl Modifier {
    /// Convert string modifier name to enum value.
    /// Accepts both "logo" and "win" for the Windows/Super key.
    /// Case-insensitive matching for user convenience.
    pub fn from_name(name: &str) -> Option<Self> {
        match name.to_lowercase().as_str() {
            "shift" => Some(Self::Shift),
            "capslock" => Some(Self::CapsLock),
            "ctrl" => Some(Self::Ctrl),
            "alt" => Some(Self::Alt),
            "logo" | "win" => Some(Self::Logo),
            "altgr" => Some(Self::AltGr),
            _ => None,
        }
    }
}

/// Parse command-line arguments into a sequence of executable commands.
/// 
/// This function processes all the different argument types and converts them into
/// a linear sequence of commands that will be executed in order. The order of
/// execution follows the order of arguments on the command line.
///
/// # Arguments
/// * `args` - Parsed command-line arguments from clap
///
/// # Returns
/// * `Ok(Vec<Command>)` - Sequence of commands to execute
/// * `Err(anyhow::Error)` - If invalid modifier names are provided
///
/// # Command Processing Order
/// 1. Text arguments (including stdin placeholder "-")
/// 2. Modifier press commands (-M)
/// 3. Modifier release commands (-m) 
/// 4. Key press commands (-P)
/// 5. Key release commands (-p)
/// 6. Type key commands (-k) - converted to press+release pairs
/// 7. Sleep commands (-s)
/// 8. Stdin flag (--stdin)
fn parse_commands(args: Args) -> anyhow::Result<Vec<Command>> {
    let mut commands = Vec::new();
    let delay = Duration::from_millis(args.delay);
    
    // Process text arguments - these can include the special "-" stdin placeholder
    for text in args.text {
        if text == "-" {
            // Special case: "-" means read from stdin at this point in the sequence
            commands.push(Command::StdinText { delay });
        } else {
            // Regular text to type with the specified delay between characters
            commands.push(Command::Text { text, delay });
        }
    }
    
    // Process modifier press commands (-M flag)
    // Each modifier name is validated and converted to the enum
    for mod_name in args.press_mod {
        let modifier = Modifier::from_name(&mod_name)
            .ok_or_else(|| anyhow::anyhow!("Invalid modifier name: {}", mod_name))?;
        commands.push(Command::ModPress(modifier));
    }
    
    // Process modifier release commands (-m flag)
    // Should typically be paired with corresponding press commands
    for mod_name in args.release_mod {
        let modifier = Modifier::from_name(&mod_name)
            .ok_or_else(|| anyhow::anyhow!("Invalid modifier name: {}", mod_name))?;
        commands.push(Command::ModRelease(modifier));
    }
    
    // Process key press commands (-P flag)
    // Keys remain pressed until explicitly released
    for key in args.press_key {
        commands.push(Command::KeyPress(key));
    }
    
    // Process key release commands (-p flag)
    // Should be paired with corresponding press commands
    for key in args.release_key {
        commands.push(Command::KeyRelease(key));
    }
    
    // Process type key commands (-k flag)
    // Each key gets converted into a press+release sequence
    for key in args.type_key {
        commands.push(Command::KeyPress(key.clone()));
        commands.push(Command::KeyRelease(key));
    }
    
    // Process sleep commands (-s flag)
    // These insert delays at specific points in the command sequence
    for sleep_ms in args.sleep {
        commands.push(Command::Sleep(Duration::from_millis(sleep_ms)));
    }
    
    // Process stdin flag (--stdin)
    // This reads all text from stdin and types it with the specified delay
    if args.stdin {
        commands.push(Command::StdinText { delay });
    }
    
    Ok(commands)
}

/// Main entry point for the wrtype application.
///
/// This function orchestrates the entire process:
/// 1. Parse command-line arguments using clap
/// 2. Validate that at least one action was specified
/// 3. Convert arguments into a command sequence
/// 4. Establish Wayland connection and virtual keyboard
/// 5. Execute all commands in sequence
/// 6. Clean up resources (automatic via RAII)
///
/// # Returns
/// * `Ok(())` - All commands executed successfully
/// * `Err(anyhow::Error)` - Various failure modes:
///   - Invalid arguments
///   - Wayland connection failure
///   - Virtual keyboard protocol not supported
///   - Command execution failure
///
/// # Exit Behavior
/// - If no actions are specified, prints usage and exits with code 1
/// - On successful completion, exits with code 0
/// - On error, anyhow handles the error display and exits with code 1
fn main() -> anyhow::Result<()> {
    // Parse command-line arguments using clap
    let args = Args::parse();
    
    // Validate that at least one action was specified
    // This matches the behavior of the original wtype implementation
    if args.text.is_empty() && 
       args.press_mod.is_empty() && 
       args.release_mod.is_empty() && 
       args.press_key.is_empty() && 
       args.release_key.is_empty() && 
       args.type_key.is_empty() &&
       args.sleep.is_empty() &&
       !args.stdin {
        eprintln!("Usage: wrtype <text-to-type>");
        std::process::exit(1);
    }
    
    // Convert command-line arguments into executable command sequence
    let commands = parse_commands(args)?;
    
    // Initialize Wayland connection and virtual keyboard protocol
    // This establishes the connection to the compositor and creates a virtual keyboard
    // that we can use to send key events
    let (connection, wayland_state) = wayland::connect_wayland()?;
    
    // Execute all commands in sequence
    // The executor handles keymap management, protocol communication, and timing
    let mut executor = CommandExecutor::new(connection, wayland_state);
    executor.execute_commands(commands)?;
    
    Ok(())
}
