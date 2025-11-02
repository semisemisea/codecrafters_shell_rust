#[allow(unused_imports)]
use std::io::{self, Write};

fn main() {
    let mut buffer = String::new();
    loop {
        print!("$ ");
        io::stdout().flush().unwrap();
        io::stdin().read_line(&mut buffer).unwrap();
        let mut words = buffer.split_whitespace();
        match words.next().unwrap() {
            "exit" => {
                let exit_code = words.next().unwrap().parse::<i32>().unwrap();
                std::process::exit(exit_code);
            }
            _ => {
                buffer.pop();
                println!("{}: command not found", buffer);
            }
        }
        buffer.clear();
    }
}
