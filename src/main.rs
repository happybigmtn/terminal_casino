mod baccarat;

mod ui;
use ui::TerminalUI;

fn main() {
    let mut terminal = TerminalUI::new();
    if let Err(e) = terminal.run() {
        eprintln!("Error: {}", e);
    }
}
