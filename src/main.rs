#[allow(unused_imports)]
use std::io::{self, Write};

fn main() {
    let mut buffer = String::new();
    loop {
        print!("$ ");
        io::stdout().flush().unwrap();
        io::stdin().read_line(&mut buffer).unwrap();
        println!("{buffer}: command not found");
    }
}
