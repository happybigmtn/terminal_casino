mod baccarat;
mod card_renderer;

mod ui;
use ui::TerminalUI;

mod tui;
use tui::RatatuiUI;

use std::env;

#[tokio::main]
async fn main() {
    let args: Vec<String> = env::args().collect();
    
    if args.len() > 1 && args[1] == "--ratatui" {
        // Use the new ratatui interface
        let mut app = RatatuiUI::new();
        if let Err(e) = app.run().await {
            eprintln!("Error: {}", e);
        }
    } else {
        // Use the original crossterm interface
        let mut terminal = TerminalUI::new();
        if let Err(e) = terminal.run() {
            eprintln!("Error: {}", e);
        }
    }
}
