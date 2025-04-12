use std::collections::HashMap;
use std::fs;
use regex::Regex;
use crate::process::parser;

#[derive(Debug)]
pub struct ProcessInfo {
    pub pid: u32,
    pub ppid: u32,
    pub name: String,
}

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
