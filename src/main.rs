// Main entry point for wrtype - a Rust implementation of wtype (xdotool type for Wayland)
//
// This module handles command-line argument parsing, command sequencing, and orchestrates
// the interaction between the Wayland virtual keyboard protocol and the XKB keymap system.

use clap::Parser;
use std::time::Duration;
use wrtype::{connect_wayland, Command, CommandExecutor, Modifier};

/// Command-line arguments structure using clap for automatic parsing and help generation.
/// This structure mirrors the original wtype interface for full compatibility.
///
/// # Examples
///
/// Basic text typing:
/// ```bash
/// wrtype "Hello, World!"
/// wrtype -- "-special-text"  # Use -- to avoid option parsing
/// ```
///
/// Keyboard shortcuts:
/// ```bash
/// wrtype -M ctrl c -m ctrl     # Ctrl+C
/// wrtype -M shift -k Tab -m shift  # Shift+Tab
/// wrtype -M ctrl -M shift t -m shift -m ctrl  # Ctrl+Shift+T
/// ```
///
/// Key sequences with timing:
/// ```bash
/// wrtype -P space -s 1000 -p space    # Hold space for 1 second
/// wrtype -k F5 -s 2000 "Page refreshed"  # F5 then wait then type
/// ```
///
/// Text with delays:
/// ```bash
/// wrtype -d 100 "Slow typing"    # 100ms between characters
/// wrtype -d 50 "Fast typing"     # 50ms between characters
/// ```
///
/// Reading from stdin:
/// ```bash
/// echo "Hello from stdin" | wrtype --stdin
/// cat file.txt | wrtype --stdin -d 10
/// ```
#[derive(Parser)]
#[command(name = "wrtype")]
#[command(about = "xdotool type for Wayland")]
#[command(version)]
pub struct Args {
    /// Text to type (use -- before text to avoid parsing as options)
    ///
    /// Multiple text arguments will be typed in sequence.
    /// Special placeholder "-" reads from stdin at that position.
    ///
    /// # Examples
    /// - `wrtype "hello world"` → Type "hello world"
    /// - `wrtype "line1" "line2"` → Type "line1" then "line2"
    /// - `wrtype "before" - "after"` → Type "before", read stdin, type "after"
    /// - `wrtype -- "-special-text"` → Type "-special-text" (avoid option parsing)
    pub text: Vec<String>,

    /// Press modifier (shift, capslock, ctrl, logo, win, alt, altgr)
    ///
    /// Modifiers remain pressed until explicitly released with -m.
    /// Can be used multiple times to press multiple modifiers.
    ///
    /// # Examples
    /// - `-M ctrl` → Press and hold Ctrl
    /// - `-M shift -M ctrl` → Press both Shift and Ctrl
    /// - `-M logo` or `-M win` → Press Super/Windows key
    #[arg(short = 'M', value_name = "MOD")]
    pub press_mod: Vec<String>,

    /// Release modifier (shift, capslock, ctrl, logo, win, alt, altgr)
    ///
    /// Should be paired with corresponding -M commands.
    /// Release order typically mirrors press order in reverse.
    ///
    /// # Examples
    /// - `-M ctrl c -m ctrl` → Ctrl+C sequence
    /// - `-M ctrl -M shift t -m shift -m ctrl` → Ctrl+Shift+T
    #[arg(short = 'm', value_name = "MOD")]
    pub release_mod: Vec<String>,

    /// Press key (using XKB key names like "Return", "Left", "space")
    ///
    /// Key remains pressed until explicitly released with -p.
    /// Useful for key combinations or sustained input.
    ///
    /// # Examples
    /// - `-P space` → Press and hold space bar
    /// - `-P Left` → Press and hold left arrow
    /// - `-P F1` → Press and hold F1 key
    #[arg(short = 'P', value_name = "KEY")]
    pub press_key: Vec<String>,

    /// Release key (using XKB key names)
    ///
    /// Should be paired with corresponding -P commands.
    /// Releases a previously pressed key.
    ///
    /// # Examples
    /// - `-P space -s 1000 -p space` → Hold space for 1 second
    /// - `-P shift -k Tab -p shift` → Shift+Tab
    #[arg(short = 'p', value_name = "KEY")]
    pub release_key: Vec<String>,

    /// Type (press and release) key in one action
    ///
    /// Equivalent to -P <key> -p <key> but more convenient.
    /// Most common way to simulate single key presses.
    ///
    /// # Examples
    /// - `-k Return` → Press Enter
    /// - `-k F5` → Press F5 (refresh)
    /// - `-k Tab` → Press Tab key
    /// - `-k Escape` → Press Escape
    #[arg(short = 'k', value_name = "KEY")]
    pub type_key: Vec<String>,

    /// Sleep for TIME milliseconds between keystrokes (default: 0)
    ///
    /// Applies to all subsequent text typing commands.
    /// Useful for applications that need time to process input.
    ///
    /// # Examples
    /// - `-d 100 "slow typing"` → 100ms between each character
    /// - `-d 0 "instant"` → No delay (default)
    /// - `-d 50 "medium speed"` → 50ms between characters
    #[arg(short = 'd', value_name = "TIME", default_value = "0")]
    pub delay: u64,

    /// Sleep for TIME milliseconds before interpreting following options
    ///
    /// Used for timing control in complex key sequences.
    /// Can be used multiple times for multiple delays.
    ///
    /// # Examples
    /// - `-P space -s 1000 -p space` → Hold space for 1 second
    /// - `-k F5 -s 2000 "Done"` → Press F5, wait 2 seconds, type "Done"
    /// - `-s 500 -k Return` → Wait 500ms then press Enter
    #[arg(short = 's', value_name = "TIME")]
    pub sleep: Vec<u64>,

    /// Read text from stdin instead of command line arguments
    ///
    /// Processes all text from stdin and types it with the
    /// specified delay between characters.
    ///
    /// # Examples
    /// - `echo "Hello" | wrtype --stdin` → Type "Hello" from pipe
    /// - `cat file.txt | wrtype --stdin -d 50` → Type file with 50ms delays
    /// - `wrtype --stdin` → Type whatever user inputs (interactive)
    #[arg(long)]
    pub stdin: bool,
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
///
/// # Examples
///
/// Simple text typing:
/// ```bash
/// wrtype "hello"
/// # → [Text { text: "hello", delay: 0ms }]
/// ```
///
/// Keyboard shortcut:
/// ```bash
/// wrtype -M ctrl c -m ctrl
/// # → [ModPress(Ctrl), ModPress(c), ModRelease(Ctrl)]
/// ```
///
/// Complex sequence with timing:
/// ```bash
/// wrtype -k F5 -s 1000 -d 100 "Page loaded"
/// # → [KeyPress("F5"), KeyRelease("F5"), Sleep(1000ms), Text { text: "Page loaded", delay: 100ms }]
/// ```
///
/// Stdin integration:
/// ```bash
/// echo "dynamic" | wrtype "Static: " - " text"
/// # → [Text { text: "Static: ", delay: 0ms }, StdinText { delay: 0ms }, Text { text: " text", delay: 0ms }]
/// ```
fn parse_commands(args: Args) -> anyhow::Result<Vec<Command>> {
    let mut commands = Vec::new();
    // Convert milliseconds to Duration once for efficiency - this delay applies to all text typing
    let delay = Duration::from_millis(args.delay);

    // PHASE 1: Process text arguments - these can include the special "-" stdin placeholder
    // Text arguments are processed in order and can be interspersed with stdin reads
    // This allows patterns like: wrtype "before" - "after" (type "before", read stdin, type "after")
    for text in args.text {
        if text == "-" {
            // Special sentinel value: "-" means read from stdin at this exact point in the sequence
            // This provides precise control over when stdin is processed relative to other text
            commands.push(Command::StdinText { delay });
        } else {
            // Regular text argument - will be typed character by character with inter-character delay
            // The delay here affects the spacing between individual characters, not words
            commands.push(Command::Text { text, delay });
        }
    }

    // PHASE 2: Process modifier press commands (-M flag)
    // Modifiers are processed as a group but maintain their command-line order
    // This allows for complex modifier combinations like: -M ctrl -M shift -M alt
    for mod_name in args.press_mod {
        // Convert string name to strongly-typed enum, with helpful error on invalid names
        // Valid names: shift, capslock, ctrl, alt, logo/win, altgr (case-insensitive)
        let modifier = Modifier::from_name(&mod_name)
            .ok_or_else(|| anyhow::anyhow!("Invalid modifier name: {}", mod_name))?;
        commands.push(Command::ModPress(modifier));
    }

    // PHASE 3: Process modifier release commands (-m flag)
    // These should typically mirror the press commands but can be in different order
    // Common pattern: press in order A,B,C then release in reverse order C,B,A for proper nesting
    for mod_name in args.release_mod {
        let modifier = Modifier::from_name(&mod_name)
            .ok_or_else(|| anyhow::anyhow!("Invalid modifier name: {}", mod_name))?;
        commands.push(Command::ModRelease(modifier));
    }

    // PHASE 4: Process key press commands (-P flag)
    // These create "sticky" key presses that remain active until explicitly released
    // Useful for key combinations or sustained input (like holding arrow keys for movement)
    for key in args.press_key {
        // Key names are passed as strings and will be validated later by the keymap system
        // This defers validation until we have access to the XKB context
        commands.push(Command::KeyPress(key));
    }

    // PHASE 5: Process key release commands (-p flag)
    // These should be paired with corresponding press commands to avoid orphaned releases
    // The command executor will handle releasing non-pressed keys gracefully
    for key in args.release_key {
        commands.push(Command::KeyRelease(key));
    }

    // PHASE 6: Process type key commands (-k flag)
    // These are convenience commands that expand to press+release pairs
    // More efficient than requiring users to specify both -P and -p for simple key taps
    for key in args.type_key {
        // Clone the key name since we need it twice for the press/release pair
        // This creates atomic key press operations that can't be interrupted
        commands.push(Command::KeyPress(key.clone()));
        commands.push(Command::KeyRelease(key));
    }

    // PHASE 7: Process sleep commands (-s flag)
    // These insert timing delays at specific points in the command sequence
    // Critical for applications that need time to process input or for precise timing
    for sleep_ms in args.sleep {
        // Convert milliseconds to Duration - these are point-in-time delays, not per-character
        commands.push(Command::Sleep(Duration::from_millis(sleep_ms)));
    }

    // PHASE 8: Process stdin flag (--stdin)
    // This adds a stdin read operation to the end of the command sequence
    // Note: this is separate from the "-" placeholder which can appear anywhere in text args
    if args.stdin {
        // Use the same character delay as regular text for consistency
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
    // PHASE 1: Parse command-line arguments using clap's derive API
    // This automatically handles --help, --version, and validates argument types
    let args = Args::parse();

    // PHASE 2: Validate that at least one action was specified
    // We need to check all possible action types to ensure the user provided meaningful input
    // This prevents the program from running with no-op behavior and matches wtype's UX
    if args.text.is_empty()
        && args.press_mod.is_empty()
        && args.release_mod.is_empty()
        && args.press_key.is_empty()
        && args.release_key.is_empty()
        && args.type_key.is_empty()
        && args.sleep.is_empty()
        && !args.stdin
    {
        // Provide a helpful error message and exit with non-zero code for shell script compatibility
        eprintln!("Usage: wrtype <text-to-type>");
        std::process::exit(1);
    }

    // PHASE 3: Convert command-line arguments into executable command sequence
    // This transforms the clap-parsed args into our internal Command representation
    // All argument validation and transformation happens here, including modifier name resolution
    let commands = parse_commands(args)?;

    // PHASE 4: Initialize Wayland connection and virtual keyboard protocol
    // This is the most complex initialization step - it involves:
    // 1. Connecting to the Wayland display server (compositor)
    // 2. Discovering available global objects via registry
    // 3. Binding to the seat (input device manager) and virtual keyboard manager
    // 4. Creating a virtual keyboard instance that can send events
    let (connection, wayland_state) = connect_wayland()?;

    // PHASE 5: Execute all commands in sequence
    // The executor is the orchestration layer that coordinates:
    // - Dynamic keymap generation and updates (for Unicode support)
    // - Wayland protocol message sending and synchronization
    // - Timing control and delay management
    // - Proper cleanup of modifier state on completion
    let mut executor = CommandExecutor::new(connection, wayland_state);
    executor.execute_commands(commands)?;

    // PHASE 6: Implicit cleanup
    // When the executor drops, it automatically releases any held modifiers
    // The Wayland connection cleanup is handled by the Drop trait implementations
    Ok(())
}
