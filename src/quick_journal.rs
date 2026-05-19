use std::env;
use std::fs;
use serde::{Deserialize, Serialize};
use chrono::Local;

#[derive(Serialize, Deserialize, Debug)]
pub struct Entry {
    pub name: String,
    pub description: String,
    pub time: String,
    pub mood: String,
}
pub fn run() {
    let args: Vec<String> = env::args().collect();
    let path_to_json = "journal.json";
    match args.get(1).map(|s| s.as_str()) {
        Some("list") => print_list(path_to_json),
        Some("help") if args.len() == 1 => {
            println!("Usage:");
            println!("  help:   cargo run help");
            println!("  Add:   cargo run add <name> <description> <mood>");
            println!("  List:  cargo run list");
            println!("  Show:  cargo run show <name>");
            println!("  remove:  cargo run remove <index> , <index>...");
        }
        
        Some("remove") if args.len() >= 3 => {
            let indices: Vec<usize> = args[2..]
                .iter()
                .map(|s| s.parse::<usize>().expect("Index must be a number"))
                .map(|i| i - 1)
                .collect();
            remove_entry(indices, path_to_json);
        }
        Some("edit") if args.len() == 4 => {
            let index = args[2].parse::<usize>().expect("Index must be a number");
            let new_description = &args[3];
            edit_entry(index, new_description, path_to_json);
        }
        Some("show") if args.len() == 3 => {
            let name = &args[2];
            read_specific_journal(name, path_to_json);
        }
        Some("add") if args.len() == 5 => {
            let name = &args[2];
            let description = &args[3];
            let mood= &args[4];
            add_journal(name, description, path_to_json, mood);
        }
        _ => {
            eprintln!("Usage:");
            eprintln!("  help:   cargo run help");
            eprintln!("  Add:   cargo run add <name> <description>");
            eprintln!("  List:  cargo run list");
            eprintln!("  Show:  cargo run show <name>");
            eprintln!("  remove:  cargo run remove <index> , <index>...");
        }
    }
}

fn add_journal(name: &str, description: &str, path_to_json: &str, mood: &str) {
    let mut journal: Vec<Entry> = fs::read_to_string(path_to_json)
        .ok()
        .and_then(|data| serde_json::from_str(&data).ok())
        .unwrap_or_default();

    journal.push(Entry {
        name: name.to_string(),
        description: description.to_string(),
        mood: mood.to_string(),
        time: Local::now().to_string(),
    });

    let updated = serde_json::to_string_pretty(&journal).unwrap();
    fs::write(path_to_json, updated).unwrap();

    println!("Added name: '{}', description: '{}', mood: '{}'.", name, description,mood);
}

fn print_list(path_to_json: &str) {
    let journal: Vec<Entry> = fs::read_to_string(path_to_json)
        .ok()
        .and_then(|data| serde_json::from_str(&data).ok())
        .unwrap_or_default();

    if journal.is_empty() {
        println!("No entries yet.");
    } else {
        for (index ,entry) in journal.iter().enumerate() {
            println!("{}) {}",index + 1, entry.name);
        }
    }
}

fn read_specific_journal(name: &str, path_to_json: &str){
    let journal: Vec<Entry> = fs::read_to_string(path_to_json)
        .ok()
        .and_then(|data| serde_json::from_str(&data).ok())
        .unwrap_or_default();

    let found: Vec<&Entry> = journal.iter().filter(|e| e.name == name).collect();
    
    if found.is_empty(){
        println!("cant find the journal");
    }else {
        for entry in found {
            println!("name: {}", entry.name);
            println!("description: {}", entry.description);
            println!("time: {}", entry.time);
        }
    }

}

fn remove_entry(mut indices: Vec<usize>, path_to_json: &str) {
    let mut journal: Vec<Entry> = fs::read_to_string(path_to_json)
        .ok()
        .and_then(|data| serde_json::from_str(&data).ok())
        .unwrap_or_default();

    indices.sort_unstable_by(|a, b| b.cmp(a));

    for &idx in &indices {
        if idx < journal.len() {
            let removed = journal.remove(idx);
            println!("Removed: {} - {}", removed.name, removed.description);
        } else {
            println!("Index {} out of range, skipping.", idx + 1);
        }
    }

    let updated = serde_json::to_string_pretty(&journal).unwrap();
    fs::write(path_to_json, updated).unwrap();
}

fn edit_entry(index: usize, new_description: &str, path_to_json: &str) {
    let mut journal: Vec<Entry> = fs::read_to_string(path_to_json)
        .ok()
        .and_then(|data| serde_json::from_str(&data).ok())
        .unwrap_or_default();
    let idx = index - 1;
    if idx >= journal.len(){
        println!("sorry the index doesnt exist")
    } else{
        let entry = &mut journal[idx];
        entry.description = new_description.to_string()
    }
    let updated = serde_json::to_string_pretty(&journal).unwrap();
fs::write(path_to_json, updated).unwrap();
println!("Updated entry {} to description: '{}'", index, new_description);
}