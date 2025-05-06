use eframe::egui::{self, CentralPanel, ScrollArea, Layout, Align, TextEdit};
use crate::process::tree::{build_process_tree, ProcessInfo};
use std::collections::HashMap;
use nix::sys::signal::{kill, Signal};
use nix::unistd::Pid;

pub struct MyApp {
    processes: Vec<ProcessInfo>,
    last_update: String,
    kill_result: Option<String>,
    filter: String,
}

impl Default for MyApp {
    fn default() -> Self {
        let (processes, time) = load_data();
        Self {
            processes,
            last_update: time,
            kill_result: None,
            filter: String::new(),
        }
    }
}

impl eframe::App for MyApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        CentralPanel::default().show(ctx, |ui| {
            ui.with_layout(Layout::top_down(Align::Min), |ui| {
                ui.heading("Linux Process Manager");
                ui.horizontal(|ui| {
                    if ui.button("🔄 Refresh").clicked() {
                        let (proc, time) = load_data();
                        self.processes = proc;
                        self.last_update = time;
                    }
                    ui.label(format!("Last updated: {}", self.last_update));
                });

                ui.add(TextEdit::singleline(&mut self.filter).hint_text("Filter by name or user..."));

                if let Some(msg) = &self.kill_result {
                    ui.colored_label(egui::Color32::RED, msg);
                }

                ui.separator();

                ScrollArea::vertical().max_height(f32::INFINITY).show(ui, |ui| {
                    egui::Grid::new("process_table")
                        .striped(true)
                        .spacing([10.0, 4.0])
                        .min_col_width(80.0)
                        .show(ui, |ui| {
                            ui.label("PID");
                            ui.label("PPID");
                            ui.label("Name");
                            ui.label("User");
                            ui.label("State");
                            ui.label("Mem (MB)");
                            ui.label("Threads");
                            ui.label("");
                            ui.end_row();

                            for proc in self.processes.iter().filter(|p| {
                                self.filter.is_empty()
                                    || p.name.to_lowercase().contains(&self.filter.to_lowercase())
                                    || p.username.to_lowercase().contains(&self.filter.to_lowercase())
                            }) {
                                ui.label(proc.pid.to_string());
                                ui.label(proc.ppid.to_string());
                                ui.label(&proc.name);
                                ui.label(&proc.username);
                                ui.label(&proc.state);
                                ui.label(format!("{:.1}", proc.memory_kb as f64 / 1024.0));
                                ui.label(proc.threads.to_string());

                                if ui.button("Kill").clicked() {
                                    match kill(Pid::from_raw(proc.pid as i32), Signal::SIGKILL) {
                                        Ok(_) => {
                                            self.kill_result = Some(format!("✅ Killed PID {}", proc.pid));
                                        }
                                        Err(e) => {
                                            self.kill_result = Some(format!("❌ Failed to kill PID {}: {}", proc.pid, e));
                                        }
                                    }
                                }

                                ui.end_row();
                            }
                        });
                });
            });
        });
    }
}

fn load_data() -> (Vec<ProcessInfo>, String) {
    let tree: HashMap<u32, Vec<ProcessInfo>> = build_process_tree();
    let mut all_processes: Vec<ProcessInfo> = tree
        .values()
        .flat_map(|v| v.iter().cloned())
        .collect();

    all_processes.sort_by_key(|p| p.pid);

    let now = chrono::Local::now().format("%H:%M:%S").to_string();
    (all_processes, now)
}
