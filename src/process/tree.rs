use std::collections::HashMap;
use std::fs;
use regex::Regex;
use crate::process::parser;
#[derive(Debug, Clone)]
pub struct ProcessInfo {
    pub pid: u32,
    pub ppid: u32,
    pub name: String,
    pub state: String,
    pub memory_kb: u64,
    pub uid: u32,
    pub username: String,
    pub start_time_ticks: u64,
    pub threads: u32,
}

/// Builds the process tree by grouping children by their PPID.
pub fn build_process_tree() -> HashMap<u32, Vec<ProcessInfo>> {
    let mut tree: HashMap<u32, Vec<ProcessInfo>> = HashMap::new();
    let re = Regex::new(r"^\d+$").unwrap();

    for entry in fs::read_dir("/proc").unwrap() {
        let entry = entry.unwrap();
        let file_name = entry.file_name();
        let name = file_name.to_string_lossy();

        if re.is_match(&name) {
            if let Some(proc_info) = parser::parse_status(&name) {
                tree.entry(proc_info.ppid)
                    .or_default()
                    .push(proc_info);
            }
        }
    }

    tree
}

/// Pretty table view for all processes (default view).
pub fn print_table(tree: &HashMap<u32, Vec<ProcessInfo>>) {
    println!(
        "{:<6} {:<6} {:<25} {:<10} {:<6} {:>8} {:>8} {:>12} {:>6}",
        "PID", "PPID", "NAME", "USER", "STATE", "MEM_MB", "THREADS", "START_TICKS", "UID"
    );

    let mut all_processes: Vec<&ProcessInfo> = tree.values().flat_map(|v| v.iter()).collect();
    all_processes.sort_by_key(|p| p.pid);

    for p in all_processes {
        println!(
            "{:<6} {:<6} {:<25.25} {:<10} {:<6} {:>8.1} {:>8} {:>12} {:>6}",
            p.pid,
            p.ppid,
            p.name,
            p.username,
            p.state,
            p.memory_kb as f64 / 1024.0,
            p.threads,
            p.start_time_ticks,
            p.uid
        );
    }
}

/// Recursive process tree view.
pub fn print_tree(tree: &HashMap<u32, Vec<ProcessInfo>>, ppid: u32, indent: usize) {
    if let Some(children) = tree.get(&ppid) {
        for child in children {
            println!(
                "{}├─ {} (PID: {}, PPID: {}, USER: {}, STATE: {}, MEM: {:.1} MB, THREADS: {})",
                " ".repeat(indent),
                child.name,
                child.pid,
                child.ppid,
                child.username,
                child.state,
                child.memory_kb as f64 / 1024.0,
                child.threads
            );
            print_tree(tree, child.pid, indent + 4);
        }
    }
}
