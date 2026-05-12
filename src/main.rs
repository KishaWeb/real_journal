use std::{env::{self}};
use std::{fs::{self}};
fn main(){
    let args: Vec<String> = env::args().collect();
    let _name: &String = &args[1];
    let json_content: Result<String, std::io::Error> = std::fs::read_to_string("journal.json");
    println!("{:?}", json_content);
}
