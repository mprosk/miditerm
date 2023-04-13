mod app;

use anyhow::Context;
use crossterm::{
    event::{DisableMouseCapture, EnableMouseCapture},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use tui::{backend::CrosstermBackend, Terminal};

/// Primary function call to start operating the TUI
///
/// Configures the terminal for TUI, runs the app, then restores the terminal and exits
pub fn run_application() -> Result<(), anyhow::Error> {
    // Set up terminal
    enable_raw_mode()?;
    let mut stdout = std::io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend).context("Unable to create TUI terminal")?;

    // Run the application
    let result = app::run_app(&mut terminal);

    // Restore terminal after application exits
    disable_raw_mode().context("Failed to disable raw mode")?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal
        .show_cursor()
        .context("Failed to restore terminal cursor")?;

    // Return the exit status
    result
}
