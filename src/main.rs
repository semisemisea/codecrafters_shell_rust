use is_executable::{self, IsExecutable};
use std::ffi::{OsStr, OsString};
use std::io::{self, BufRead, BufReader, Write};
use std::path;
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
        s.insert("pwd");
        s.insert("cd");
        s
    })
}

fn path_executables() -> std::io::Result<HashMap<OsString, path::PathBuf>> {
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

fn main() -> io::Result<()> {
    let mut buffer = String::new();
    let path_env_exec = path_executables().unwrap();
    let mut curr_dir = path::absolute(path::Path::new(".")).unwrap();
    loop {
        print!("$ ");
        io::stdout().flush().unwrap();
        io::stdin().read_line(&mut buffer).unwrap();
        let mut words = shlex::split(&buffer).unwrap().into_iter();
        match words.next().unwrap().as_str() {
            "exit" => {
                let exit_code = words
                    .next()
                    .unwrap_or("0".to_string())
                    .parse::<i32>()
                    .unwrap();
                std::process::exit(exit_code);
            }
            "echo" => {
                for word in words {
                    print!("{word} ");
                }
                println!()
            }
            "type" => match words.next() {
                Some(obj) if built_ins().contains(obj.as_str()) => {
                    println!("{obj} is a shell builtin");
                }
                Some(exec) if path_env_exec.contains_key(OsStr::new(&exec)) => {
                    println!(
                        "{exec} is {}",
                        path_env_exec.get(OsStr::new(&exec)).unwrap().display()
                    )
                }
                Some(other) => {
                    println!("{other}: not found");
                }
                None => {
                    println!("Usage: type <command>");
                }
            },
            "pwd" => {
                println!("{}", curr_dir.display());
            }
            "cd" => {
                let change_to = words.next().unwrap();
                if let Some(rest) = change_to.strip_prefix("~") {
                    let mut target = std::env::home_dir().clone().unwrap();
                    let target = target.as_mut_os_string();
                    target.push(rest);
                    // println!("{}", target.display());
                    curr_dir = path::PathBuf::from(&target);
                } else {
                    let dir = path::Path::new(&change_to);
                    if dir.is_absolute() {
                        if !dir.exists() {
                            println!("cd: {}: No such file or directory", dir.display());
                        }
                        curr_dir = dir.to_path_buf();
                    } else {
                        let target = path_clean::clean(curr_dir.join(dir));
                        if target.exists() {
                            curr_dir = target;
                        } else {
                            println!("cd: {}: No such file or directory", target.display())
                        };
                    }
                }
            }
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
