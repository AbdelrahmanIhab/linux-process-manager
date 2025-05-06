use eframe::egui::{self, CentralPanel, ScrollArea, RichText, Color32, Ui};
use crate::process::tree::{build_process_tree, ProcessInfo};
use nix::sys::signal::{kill, Signal};
use nix::unistd::Pid;
use std::collections::{HashMap, HashSet};
use std::fs;
use chrono::Local;

#[derive(PartialEq, Eq)]
enum SortKey {
    PID,
    CPU,
    Memory,
    Name,
}

pub struct MyApp {
    processes: Vec<ProcessInfo>,
    process_tree: HashMap<u32, Vec<ProcessInfo>>,
    last_update: String,
    filter_text: String,
    prev_total_cpu: u64,
    prev_process_cpu: HashMap<u32, u64>,
    auto_refresh: bool,
    show_tree: bool,
    last_refresh_time: std::time::Instant,
    expanded: HashSet<u32>,
    sort_key: SortKey,
    descending: bool,
    selected_pids: HashSet<u32>,
}

impl Default for MyApp {
    fn default() -> Self {
        let tree = build_process_tree();
        let flat = tree.values().flat_map(|v| v.clone()).collect();
        let now = Local::now().format("%H:%M:%S").to_string();

        Self {
            processes: flat,
            process_tree: tree,
            last_update: now,
            filter_text: String::new(),
            prev_total_cpu: 0,
            prev_process_cpu: HashMap::new(),
            auto_refresh: false,
            show_tree: false,
            last_refresh_time: std::time::Instant::now(),
            expanded: HashSet::new(),
            sort_key: SortKey::PID,
            descending: false,
            selected_pids: HashSet::new(),
        }
    }
}

impl eframe::App for MyApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        if self.auto_refresh && self.last_refresh_time.elapsed().as_secs_f32() >= 1.0 {
            let (flat, tree, time, total, proc_cpu) =
                load_data(self.prev_process_cpu.clone(), self.prev_total_cpu);
            self.processes = flat;
            self.process_tree = tree;
            self.last_update = time;
            self.prev_process_cpu = proc_cpu;
            self.prev_total_cpu = total;
            self.last_refresh_time = std::time::Instant::now();
        }

        CentralPanel::default().show(ctx, |ui| {
            ui.heading("Linux Process Manager");

            ui.horizontal(|ui| {
                if ui.button("🔄 Refresh").clicked() {
                    let (flat, tree, time, total, proc_cpu) =
                        load_data(self.prev_process_cpu.clone(), self.prev_total_cpu);
                    self.processes = flat;
                    self.process_tree = tree;
                    self.last_update = time;
                    self.prev_process_cpu = proc_cpu;
                    self.prev_total_cpu = total;
                    self.last_refresh_time = std::time::Instant::now();
                }

                ui.checkbox(&mut self.auto_refresh, "Auto-refresh");
                ui.checkbox(&mut self.show_tree, "Tree View");
                ui.label(format!("Updated: {}", self.last_update));
            });

            ui.horizontal(|ui| {
                ui.label("Filter:");
                ui.text_edit_singleline(&mut self.filter_text);
                if ui.button("❌ Kill Selected").clicked() {
                    for &pid in &self.selected_pids {
                        let _ = kill(Pid::from_raw(pid as i32), Signal::SIGKILL);
                    }
                    self.selected_pids.clear();
                }
            });

            ui.separator();

            ScrollArea::vertical().show(ui, |ui| {
                egui::Grid::new("proc_table")
                    .striped(true)
                    .spacing([10.0, 4.0])
                    .show(ui, |ui| {
                        let pid_header = match self.sort_key {
                            SortKey::PID => if self.descending { "PID ▼" } else { "PID ▲" },
                            _ => "PID",
                        };
                        let name_header = match self.sort_key {
                            SortKey::Name => if self.descending { "Name ▼" } else { "Name ▲" },
                            _ => "Name",
                        };
                        let mem_header = match self.sort_key {
                            SortKey::Memory => if self.descending { "Mem (MB) ▼" } else { "Mem (MB) ▲" },
                            _ => "Mem (MB)",
                        };
                        let cpu_header = match self.sort_key {
                            SortKey::CPU => if self.descending { "CPU % ▼" } else { "CPU % ▲" },
                            _ => "CPU %",
                        };

                        ui.label(""); // Checkbox
                        if ui.button(pid_header).clicked() { self.toggle_sort(SortKey::PID); }
                        ui.label("PPID");
                        if ui.button(name_header).clicked() { self.toggle_sort(SortKey::Name); }
                        ui.label("User");
                        ui.label("State");
                        if ui.button(mem_header).clicked() { self.toggle_sort(SortKey::Memory); }
                        ui.label("Threads");
                        if ui.button(cpu_header).clicked() { self.toggle_sort(SortKey::CPU); }
                        ui.label("❌");
                        ui.label("⏸️");
                        ui.end_row();

                        if self.show_tree {
                            if let Some(roots) = self.process_tree.get(&0) {
                                for root in roots {
                                    draw_tree_row(
                                        ui,
                                        root,
                                        &self.process_tree,
                                        &mut self.expanded,
                                        &mut self.selected_pids,
                                        0,
                                        &self.filter_text,
                                    );
                                }
                            }
                        } else {
                            let mut rows: Vec<_> = self.processes.iter()
                                .filter(|p| p.name.contains(&self.filter_text)
                                    || p.username.contains(&self.filter_text)
                                    || p.pid.to_string().contains(&self.filter_text))
                                .collect();

                            match self.sort_key {
                                SortKey::PID => rows.sort_by_key(|p| p.pid),
                                SortKey::CPU => rows.sort_by(|a, b| a.cpu_usage.partial_cmp(&b.cpu_usage).unwrap()),
                                SortKey::Memory => rows.sort_by_key(|p| p.memory_kb),
                                SortKey::Name => rows.sort_by_key(|p| p.name.clone()),
                            }
                            if self.descending { rows.reverse(); }

                            for proc in rows {
                                draw_row(ui, proc, &mut self.selected_pids);
                            }
                        }
                    });
            });
        });

        if self.auto_refresh {
            ctx.request_repaint_after(std::time::Duration::from_millis(500));
        }
    }
}

impl MyApp {
    fn toggle_sort(&mut self, key: SortKey) {
        if self.sort_key == key {
            self.descending = !self.descending;
        } else {
            self.sort_key = key;
            self.descending = false;
        }
    }
}

fn draw_tree_row(
    ui: &mut Ui,
    proc: &ProcessInfo,
    tree: &HashMap<u32, Vec<ProcessInfo>>,
    expanded: &mut HashSet<u32>,
    selected: &mut HashSet<u32>,
    depth: usize,
    filter: &str,
) {
    if !proc.name.contains(filter)
        && !proc.username.contains(filter)
        && !proc.pid.to_string().contains(filter)
    {
        return;
    }

    let mut checked = selected.contains(&proc.pid);
    ui.checkbox(&mut checked, "");
    if checked { selected.insert(proc.pid); } else { selected.remove(&proc.pid); }

    if tree.get(&proc.pid).is_some() {
        if ui.button(if expanded.contains(&proc.pid) { "▼" } else { "▶" }).clicked() {
            if expanded.contains(&proc.pid) {
                expanded.remove(&proc.pid);
            } else {
                expanded.insert(proc.pid);
            }
        }
    } else {
        ui.label("  ");
    }

    draw_process_cells(ui, proc);

    if expanded.contains(&proc.pid) {
        if let Some(children) = tree.get(&proc.pid) {
            for child in children {
                draw_tree_row(ui, child, tree, expanded, selected, depth + 1, filter);
            }
        }
    }
}

fn draw_row(ui: &mut Ui, proc: &ProcessInfo, selected: &mut HashSet<u32>) {
    let mut checked = selected.contains(&proc.pid);
    ui.checkbox(&mut checked, "");
    if checked { selected.insert(proc.pid); } else { selected.remove(&proc.pid); }

    draw_process_cells(ui, proc);
}

fn draw_process_cells(ui: &mut Ui, proc: &ProcessInfo) {
    ui.label(proc.pid.to_string());
    ui.label(proc.ppid.to_string());
    ui.label(RichText::new(&proc.name).color(if proc.cpu_usage > 50.0 { Color32::RED } else { Color32::WHITE }));
    ui.label(&proc.username);
    ui.label(RichText::new(&proc.state).color(match proc.state.as_str() {
        "S" => Color32::GRAY, "R" => Color32::GREEN, _ => Color32::LIGHT_BLUE,
    }));
    ui.label(format!("{:.1}", proc.memory_kb as f64 / 1024.0));
    ui.label(proc.threads.to_string());
    ui.label(format!("{:.1}", proc.cpu_usage));
    ui.label(""); // kill btn optional
    ui.label(""); // stop btn optional
    ui.end_row();
}

fn load_data(
    prev_cpu_map: HashMap<u32, u64>,
    prev_total_time: u64,
) -> (Vec<ProcessInfo>, HashMap<u32, Vec<ProcessInfo>>, String, u64, HashMap<u32, u64>) {
    let tree = build_process_tree();
    let mut flat: Vec<ProcessInfo> = tree.values().flat_map(|v| v.clone()).collect();
    flat.sort_by_key(|p| p.pid);

    let total_cpu_time = get_total_cpu_time();
    let mut proc_cpu_map = HashMap::new();

    for p in flat.iter_mut() {
        if let Ok(stat_line) = std::fs::read_to_string(format!("/proc/{}/stat", p.pid)) {
            let fields: Vec<&str> = stat_line.split_whitespace().collect();
            if fields.len() > 15 {
                let utime: u64 = fields[13].parse().unwrap_or(0);
                let stime: u64 = fields[14].parse().unwrap_or(0);
                let total = utime + stime;
                proc_cpu_map.insert(p.pid, total);

                let prev = prev_cpu_map.get(&p.pid).copied().unwrap_or(0);
                let delta_proc = total.saturating_sub(prev);
                let delta_total = total_cpu_time.saturating_sub(prev_total_time);

                p.cpu_usage = if delta_total > 0 {
                    (delta_proc as f32 / delta_total as f32) * 100.0
                } else {
                    0.0
                };
            }
        }
    }

    let now = Local::now().format("%H:%M:%S").to_string();
    (flat, tree, now, total_cpu_time, proc_cpu_map)
}

fn get_total_cpu_time() -> u64 {
    if let Ok(stat) = fs::read_to_string("/proc/stat") {
        if let Some(cpu_line) = stat.lines().find(|l| l.starts_with("cpu ")) {
            let fields: Vec<&str> = cpu_line.split_whitespace().skip(1).collect();
            return fields.iter().filter_map(|v| v.parse::<u64>().ok()).sum();
        }
    }
    0
}
