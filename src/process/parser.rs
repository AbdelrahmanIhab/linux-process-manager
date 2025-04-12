use std::fs::File;
use std::io::{self, BufRead};
use crate::process::tree::ProcessInfo;

pub fn parse_status(pid: &str) -> Option<ProcessInfo> {
    let path = format!("/proc/{}/status", pid);
    let file = File::open(path).ok()?;
    let reader = io::BufReader::new(file);

    let mut name = String::new();
    let mut ppid = 0;

    for line in reader.lines().flatten() {
        if line.starts_with("Name:") {
            name = line.split_whitespace().nth(1)?.to_string();
        }
        if line.starts_with("PPid:") {
            ppid = line.split_whitespace().nth(1)?.parse().ok()?;
        }
    }

    Some(ProcessInfo {
        pid: pid.parse().ok()?,
        ppid,
        name,
    })
}
