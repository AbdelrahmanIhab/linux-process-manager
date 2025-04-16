use std::process::Command;
use std::io;

pub fn kill_process(pid: u32) -> io::Result<()> {
    Command::new("kill")
        .arg("-9")
        .arg(pid.to_string())
        .status()
        .map(|_| ())
}
