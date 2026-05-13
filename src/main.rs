use std::env;
use std::fs;
use std::time::SystemTime;
use serde::{Deserialize, Serialize};
use serde_json;
use chrono;
#[derive(Serialize, Deserialize, Debug)]
struct Entry {
    name: String,
    description: String,
    time: String,
}

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() != 3 {
        eprintln!("Usage: cargo run <name> <description>");
        return;
    }
    let path_to_json = "journal.json";
    let name = &args[1];
    let description = &args[2];
    add_journal(name, description, path_to_json);
}

fn add_journal(name: &str, description: &str, path_to_json: &str) {
    let mut journal: Vec<Entry> = fs::read_to_string(path_to_json)
        .ok()
        .and_then(|data| serde_json::from_str(&data).ok())
        .unwrap_or_default();

    journal.push(Entry {
        name: name.to_string(),
        description: description.to_string(),
        time: chrono::Local::now().to_string(),
    });

    let updated = serde_json::to_string_pretty(&journal).unwrap();
    fs::write(path_to_json, updated).unwrap();

    println!("Added name: '{}', description: '{}'. New journal: {:#?}", name, description, journal);
}
