use std::env;
use std::fs;
use serde::{Deserialize, Serialize};
use chrono::Local;

#[derive(Serialize, Deserialize, Debug)]
struct Entry {
    name: String,
    description: String,
    time: String,
}

fn main() {
    let args: Vec<String> = env::args().collect();
    let path_to_json = "journal.json";

    match args.get(1).map(|s| s.as_str()) {
        Some("list") => print_list(path_to_json),
        Some(_) if args.len() == 3 => {
            let name = &args[1];
            let description = &args[2];
            add_journal(name, description, path_to_json);
        }
        _ => {
            eprintln!("Usage:");
            eprintln!("  Add:  cargo run <name> <description>");
            eprintln!("  List: cargo run list");
        }
    }
}

fn add_journal(name: &str, description: &str, path_to_json: &str) {
    let mut journal: Vec<Entry> = fs::read_to_string(path_to_json)
        .ok()
        .and_then(|data| serde_json::from_str(&data).ok())
        .unwrap_or_default();

    journal.push(Entry {
        name: name.to_string(),
        description: description.to_string(),
        time: Local::now().to_string(),
    });

    let updated = serde_json::to_string_pretty(&journal).unwrap();
    fs::write(path_to_json, updated).unwrap();

    println!("Added name: '{}', description: '{}'.", name, description);
}

fn print_list(path_to_json: &str) {
    let journal: Vec<Entry> = fs::read_to_string(path_to_json)
        .ok()
        .and_then(|data| serde_json::from_str(&data).ok())
        .unwrap_or_default();

    if journal.is_empty() {
        println!("No entries yet.");
    } else {
        for entry in &journal {
            println!("{}", entry.name);
        }
    }
}