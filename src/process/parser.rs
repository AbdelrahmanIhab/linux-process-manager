use std::fs::{self, File};
use std::io::{self, BufRead, Read};
use crate::process::tree::ProcessInfo;

/// Parses `/proc/<pid>/status` and `/proc/<pid>/stat` to extract process details
pub fn parse_status(pid: &str) -> Option<ProcessInfo> {
    let status_path = format!("/proc/{}/status", pid);
    let file = File::open(status_path).ok()?;
    let reader = io::BufReader::new(file);

    let mut name = String::new();
    let mut ppid = 0;
    let mut state = String::new();
    let mut memory_kb = 0;
    let mut uid = 0;
    let mut threads = 0;

    for line in reader.lines().flatten() {
        if line.starts_with("Name:") {
            name = line.split_whitespace().nth(1)?.to_string();
        } else if line.starts_with("PPid:") {
            ppid = line.split_whitespace().nth(1)?.parse().ok()?;
        } else if line.starts_with("State:") {
            state = line.split_whitespace().nth(1)?.to_string();
        } else if line.starts_with("VmRSS:") {
            memory_kb = line.split_whitespace().nth(1)?.parse().ok()?;
        } else if line.starts_with("Uid:") {
            uid = line.split_whitespace().nth(1)?.parse().ok()?;
        } else if line.starts_with("Threads:") {
            threads = line.split_whitespace().nth(1)?.parse().ok()?;
        }
    }

    // Read /proc/<pid>/stat for start time
    let stat_path = format!("/proc/{}/stat", pid);
    let stat_file = File::open(stat_path).ok()?;
    let mut stat_reader = io::BufReader::new(stat_file);
    let mut stat_line = String::new();
    stat_reader.read_line(&mut stat_line).ok()?;
    let stat_fields: Vec<&str> = stat_line.split_whitespace().collect();
    let start_time_ticks = stat_fields.get(21)?.parse().ok()?; // 22nd field

    let username = get_username(uid);

    Some(ProcessInfo {
        pid: pid.parse().ok()?,
        ppid,
        name,
        state,
        memory_kb,
        uid,
        username,
        start_time_ticks,
        threads,
    })
}

/// Converts UID to username by parsing /etc/passwd
pub fn get_username(uid: u32) -> String {
    if let Ok(passwd) = fs::read_to_string("/etc/passwd") {
        for line in passwd.lines() {
            let parts: Vec<&str> = line.split(':').collect();
            if parts.len() > 2 {
                if let Ok(file_uid) = parts[2].parse::<u32>() {
                    if file_uid == uid {
                        return parts[0].to_string(); // username
                    }
                }
            }
        }
    }
    uid.to_string() // fallback: show UID as string
}
