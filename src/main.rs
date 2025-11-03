use is_executable::{self, IsExecutable};
use std::ffi::{OsStr, OsString};
use std::io::{self, BufRead, BufReader, Write};
use std::path::PathBuf;
use std::process::{Command, Stdio};
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
                let exit_code = words.next().unwrap_or("0").parse::<i32>().unwrap();
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
                        "{exec} is {}",
                        path_env_exec.get(OsStr::new(exec)).unwrap().display()
                    )
                }
                other => {
                    println!("{other}: not found");
                }
            },
            executable if path_env_exec.contains_key(OsStr::new(executable)) => {
                let args = words.collect::<Vec<_>>();
                let mut command = Command::new(executable);
                command.args(args);
                command.stdout(Stdio::piped());
                command.stderr(Stdio::piped());
                if let Ok(mut child) = command.spawn() {
                    let stdout = child.stdout.take().expect("Failed to open stdout");
                    let stderr = child.stderr.take().expect("Failed to open stderr");
                    let mut stdout_reader = BufReader::new(stdout);
                    let mut stderr_reader = BufReader::new(stderr);
                    let stdout_handle = std::thread::spawn(move || {
                        let mut line = String::new();
                        while let Ok(bytes) = stdout_reader.read_line(&mut line) {
                            if bytes == 0 {
                                break;
                            }
                            print!("{line}");
                            line.clear();
                        }
                    });
                    let stderr_handle = std::thread::spawn(move || {
                        let mut line = String::new();
                        while let Ok(bytes) = stderr_reader.read_line(&mut line) {
                            if bytes == 0 {
                                break;
                            }
                            print!("{line}");
                            line.clear();
                        }
                    });
                    stdout_handle.join().unwrap();
                    stderr_handle.join().unwrap();
                    let _status = child.wait().unwrap();
                }
            }
            _ => {
                println!("{}: command not found", buffer.trim());
            }
        }
        buffer.clear();
    }
}
