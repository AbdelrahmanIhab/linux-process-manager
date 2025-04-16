use std::collections::HashMap;
use regex::Regex;
use std::fs;

#[derive(Debug)]
pub struct ProcessInfo {
    pub pid: u32,
    pub ppid: u32,
    pub name: String,
}

pub fn build_process_tree() -> HashMap<u32, Vec<ProcessInfo>> {
    let mut tree: HashMap<u32, Vec<ProcessInfo>> = HashMap::new();
    let re = Regex::new(r"^\d+$").unwrap();

    if let Ok(entries) = fs::read_dir("/proc") {
        for entry in entries.flatten() {
            let file_name = entry.file_name();
            let name = file_name.to_string_lossy();

            if re.is_match(&name) {
                if let Some(proc_info) = parse_status(&name) {
                    tree.entry(proc_info.ppid)
                        .or_default()
                        .push(proc_info);
                }
            }
        }
    }
    tree
}

fn parse_status(pid_str: &str) -> Option<ProcessInfo> {
    let status_path = format!("/proc/{}/status", pid_str);
    let content = fs::read_to_string(status_path).ok()?;

    let mut name = None;
    let mut pid = None;
    let mut ppid = None;

    for line in content.lines() {
        if line.starts_with("Name:") {
            name = Some(line[6..].trim().to_string());
        } else if line.starts_with("Pid:") {
            pid = Some(line[5..].trim().parse().ok()?);
        } else if line.starts_with("PPid:") {
            ppid = Some(line[6..].trim().parse().ok()?);
        }

        if name.is_some() && pid.is_some() && ppid.is_some() {
            break;
        }
    }

    Some(ProcessInfo {
        name: name?,
        pid: pid?,
        ppid: ppid?,
    })
}

pub fn print_tree(tree: &HashMap<u32, Vec<ProcessInfo>>, ppid: u32, indent: usize) {
    if let Some(children) = tree.get(&ppid) {
        for child in children {
            println!(
                "{}├─ {} ({})",
                " ".repeat(indent),
                child.name,
                child.pid
            );
            print_tree(tree, child.pid, indent + 4);
        }
    }
}
