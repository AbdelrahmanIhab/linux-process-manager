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
    pub cpu_usage: f32, // ✅ Added
}

pub fn build_process_tree() -> HashMap<u32, Vec<ProcessInfo>> {
    let mut tree: HashMap<u32, Vec<ProcessInfo>> = HashMap::new();
    let re = Regex::new(r"^\d+$").unwrap();

    for entry in fs::read_dir("/proc").unwrap() {
        let entry = entry.unwrap();
        let file_name = entry.file_name();
        let name = file_name.to_string_lossy();

        if re.is_match(&name) {
            if let Some(mut proc_info) = parser::parse_status(&name) {
                proc_info.cpu_usage = 0.0; // ✅ Initialize
                tree.entry(proc_info.ppid)
                    .or_default()
                    .push(proc_info);
            }
        }
    }

    tree
}
