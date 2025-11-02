#[allow(unused_imports)]
use std::io::{self, Write};
use std::{collections::HashSet, sync::OnceLock};

fn built_in() -> &'static HashSet<&'static str> {
    static SET: OnceLock<HashSet<&'static str>> = OnceLock::new();
    SET.get_or_init(|| {
        let mut s = HashSet::new();
        s.insert("exit");
        s.insert("echo");
        s.insert("type");
        s
    })
}

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
            "echo" => {
                let content = buffer.strip_prefix("echo").unwrap();
                let content = content.trim();
                println!("{}", content);
            }
            "type" => match words.next().unwrap() {
                obj if built_in().contains(obj) => {
                    println!("{obj} is a shell builtin");
                }
                other => {
                    println!("{other}: not found");
                }
            },
            _ => {
                println!("{}: command not found", buffer.trim());
            }
        }
        buffer.clear();
    }
}
