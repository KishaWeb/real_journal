use std::{env::{self}};
use std::{fs::{self}};
use serde_json::{Value, json};
fn main(){
    let args: Vec<String> = env::args().collect();
    if args.len() < 3{
        eprint!("there cant be more then 3 args -- name discription")
    }
    let _name: &String = &args[1];
    add_journal(_name);
}

fn add_journal(name: &str){
    let mut journal: Vec<String> = std::fs::read_to_string("journal.json")
        .ok()
        .and_then(|content| serde_json::from_str(&content).ok())
        .unwrap_or_default(); 

    journal.push(name.to_string());

    let updated_content = serde_json::to_string_pretty(&journal)
        .expect("failed to idk");
    
    fs::write("journal.json", updated_content)
        .expect("failed to write");
}