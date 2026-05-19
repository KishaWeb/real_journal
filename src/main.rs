mod quick_journal;
mod tui;
use std::env;

fn main() {
    let args: Vec<String> = env::args().collect();
    
    match args.get(1).map(|s| s.as_str()) {
        _ => {
            quick_journal::run();
        }
    }
}