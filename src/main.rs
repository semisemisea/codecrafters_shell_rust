use is_executable::{self, IsExecutable};
use std::ffi::{OsStr, OsString};
use std::io::{self, Write};
use std::path::PathBuf;
use std::{
    collections::{HashMap, HashSet},
    sync::OnceLock,
};

fn built_ins() -> &'static HashSet<&'static str> {
    static SET: OnceLock<HashSet<&'static str>> = OnceLock::new();
    SET.get_or_init(|| {
        let mut s = HashSet::new();
        s.insert("exit");
        s.insert("echo");
        s.insert("type");
        s
    })
}

fn path_executables() -> std::io::Result<HashMap<OsString, PathBuf>> {
    let mut map = HashMap::new();
    if let Some(ref path) = std::env::var_os("PATH") {
        for path in std::env::split_paths(path) {
            if let Ok(path) = std::fs::read_dir(path) {
                for entry in path {
                    let dir = entry?;
                    if dir.path().is_executable() {
                        let name = dir.file_name();
                        let path = dir.path();
                        map.entry(name).or_insert(path);
                    }
                }
            }
        }
    }
    Ok(map)
}

fn main() {
    let mut buffer = String::new();
    let path_env_exec = path_executables().unwrap();
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
                obj if built_ins().contains(obj) => {
                    println!("{obj} is a shell builtin");
                }
                exec if path_env_exec.contains_key(OsStr::new(exec)) => {
                    println!(
                        "{exec} is {:?}",
                        path_env_exec.get(OsStr::new(exec)).unwrap()
                    )
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
