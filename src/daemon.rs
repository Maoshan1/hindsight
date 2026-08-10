use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::io::Write;
use std::io::{BufRead, BufReader};
use std::os::unix::net::{UnixListener, UnixStream};
use std::path::PathBuf;

use crate::db;

fn socket_path() -> PathBuf {
    let home = dirs::home_dir().expect("cannot find home dir");
    let dir = home.join(".hindsight");
    std::fs::create_dir_all(&dir).ok();
    dir.join("hindsight.sock")
}

#[derive(Deserialize, Serialize)]
struct HookMessage {
    command: String,
    cwd: String,
    exit: Option<i32>,
    duration: Option<u64>,
}

pub fn run() -> Result<()> {
    let path = socket_path();

    // Remove stale socket
    if path.exists() {
        std::fs::remove_file(&path)?;
    }

    let listener = UnixListener::bind(&path)?;
    eprintln!("hindsight daemon listening on {}", path.display());

    let conn = db::open()?;

    for stream in listener.incoming() {
        match stream {
            Ok(stream) => {
                let reader = BufReader::new(stream);
                for line in reader.lines() {
                    let line = match line {
                        Ok(l) => l,
                        Err(_) => continue,
                    };
                    match serde_json::from_str::<HookMessage>(&line) {
                        Ok(msg) => {
                            if let Err(e) =
                                db::insert(&conn, &msg.command, &msg.cwd, msg.exit, msg.duration)
                            {
                                eprintln!("hindsight daemon: insert error: {}", e);
                            }
                        }
                        Err(e) => {
                            eprintln!("hindsight daemon: parse error: {} (line: {})", e, line);
                        }
                    }
                }
            }
            Err(e) => {
                eprintln!("hindsight daemon: connection error: {}", e);
            }
        }
    }

    Ok(())
}

#[allow(dead_code)]
pub fn send_to_daemon(
    command: &str,
    cwd: &str,
    exit: Option<i32>,
    duration: Option<u64>,
) -> Result<()> {
    let path = socket_path();
    if !path.exists() {
        anyhow::bail!("daemon not running");
    }

    let msg = HookMessage {
        command: command.to_string(),
        cwd: cwd.to_string(),
        exit,
        duration,
    };

    let data = serde_json::to_string(&msg)?;
    let mut stream = UnixStream::connect(path)?;
    stream.write_all(data.as_bytes())?;
    stream.write_all(b"\n")?;

    Ok(())
}
