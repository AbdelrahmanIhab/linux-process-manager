use eframe::egui;
use linux_process_manager::process::{tree, control};
use std::collections::{HashMap, HashSet};

#[derive(PartialEq)]
enum ViewMode {
    Tree,
    Table,
}

pub struct ProcessTreeApp {
    tree: HashMap<u32, Vec<tree::ProcessInfo>>,
    last_update: std::time::Instant,
    auto_refresh: bool,
    refresh_interval: u64,
    collapsed_nodes: HashSet<u32>,
    view_mode: ViewMode,
    current_page: usize,
    processes_per_page: usize,
    all_processes: Vec<(u32, tree::ProcessInfo)>,
}

impl Default for ProcessTreeApp {
    fn default() -> Self {
        let tree = tree::build_process_tree();
        let all_processes = Self::collect_all_processes(&tree);

        Self {
            tree,
            last_update: std::time::Instant::now(),
            auto_refresh: true,
            refresh_interval: 5,
            collapsed_nodes: HashSet::new(),
            view_mode: ViewMode::Table,
            current_page: 0,
            processes_per_page: 100,
            all_processes,
        }
    }
}

impl ProcessTreeApp {
    fn collect_all_processes(tree: &HashMap<u32, Vec<tree::ProcessInfo>>) -> Vec<(u32, tree::ProcessInfo)> {
        let mut all_processes = Vec::new();
        for (ppid, children) in tree {
            for child in children {
                all_processes.push((*ppid, tree::ProcessInfo {
                    pid: child.pid,
                    ppid: child.ppid,
                    name: child.name.clone(),
                }));
            }
        }
        all_processes.sort_by_key(|(_, child)| child.pid);
        all_processes
    }

    fn update_process_cache(&mut self) {
        self.all_processes = Self::collect_all_processes(&self.tree);
        self.current_page = 0;
    }

    fn render_process_table(&mut self, ui: &mut egui::Ui) {
        let total_pages = (self.all_processes.len() + self.processes_per_page - 1) / self.processes_per_page;
        let start_idx = self.current_page * self.processes_per_page;
        let end_idx = (self.current_page + 1) * self.processes_per_page;
        let processes_to_show = &self.all_processes[start_idx..end_idx.min(self.all_processes.len())];

        ui.horizontal(|ui| {
            ui.colored_label(
                egui::Color32::from_rgb(70, 130, 180),
                format!(
                    "Showing processes {} to {} of {} (Page {}/{})",
                    start_idx + 1,
                    end_idx.min(self.all_processes.len()),
                    self.all_processes.len(),
                    self.current_page + 1,
                    total_pages.max(1)
                )
            );
        });

        ui.horizontal(|ui| {
            ui.visuals_mut().widgets.inactive.bg_fill = egui::Color32::from_rgb(70, 130, 180);
            ui.visuals_mut().widgets.hovered.bg_fill = egui::Color32::from_rgb(70, 130, 180);

            if ui.button("|<").clicked() && self.current_page > 0 {
                self.current_page = 0;
            }
            if ui.button("<").clicked() && self.current_page > 0 {
                self.current_page -= 1;
            }
            if ui.button(">").clicked() && self.current_page < total_pages - 1 {
                self.current_page += 1;
            }
            if ui.button(">|").clicked() && self.current_page < total_pages - 1 {
                self.current_page = total_pages - 1;
            }

            ui.add(
                egui::DragValue::new(&mut self.current_page)
                    .clamp_range(0..=total_pages.saturating_sub(1))
                    .prefix("Page ")
            );
        });

        egui::Grid::new("process_table")
            .num_columns(4)
            .striped(true)
            .show(ui, |ui| {
                ui.label(egui::RichText::new("PID").color(egui::Color32::from_rgb(70, 130, 180)).strong());
                ui.label(egui::RichText::new("Name").color(egui::Color32::from_rgb(70, 130, 180)).strong());
                ui.label(egui::RichText::new("PPID").color(egui::Color32::from_rgb(70, 130, 180)).strong());
                ui.label(egui::RichText::new("Action").color(egui::Color32::from_rgb(70, 130, 180)).strong());
                ui.end_row();

                for (ppid, child) in processes_to_show {
                    ui.label(child.pid.to_string());
                    ui.label(&child.name);
                    ui.label(ppid.to_string());

                    if ui.button(
                        egui::RichText::new("Kill").color(egui::Color32::from_rgb(178, 34, 34))
                    ).clicked() {
                        let _ = control::kill_process(child.pid);
                    }

                    ui.end_row();
                }
            });
    }

    fn render_process_tree(&mut self, ui: &mut egui::Ui, ppid: u32, indent: usize) {
        let empty_vec = Vec::new();
        let children = self.tree.get(&ppid).unwrap_or(&empty_vec);
    
        // Step 1: Process all children
        let mut children_to_render = vec![];
    
        for child in children.iter() {
            let has_children = self.tree.contains_key(&child.pid);
    
            ui.horizontal(|ui| {
                ui.add_space(indent as f32);
    
                if has_children {
                    let arrow = if self.collapsed_nodes.contains(&child.pid) {
                        ">"
                    } else {
                        "v"
                    };
    
                    if ui.button(arrow).clicked() {
                        if self.collapsed_nodes.contains(&child.pid) {
                            self.collapsed_nodes.remove(&child.pid);
                        } else {
                            self.collapsed_nodes.insert(child.pid);
                        }
                    }
                } else {
                    ui.label(" ");
                }
    
                ui.label(format!("{} (PID: {})", child.name, child.pid));
    
                if ui.button(
                    egui::RichText::new("Kill").color(egui::Color32::from_rgb(178, 34, 34))
                ).clicked() {
                    let _ = control::kill_process(child.pid);
                }
            });
    
            // Collect the children for recursive rendering later
            if has_children && !self.collapsed_nodes.contains(&child.pid) {
                children_to_render.push(child.pid);
            }
        }
    
        // Step 2: Render children recursively (outside the loop to avoid conflicting borrows)
        for child_pid in children_to_render.iter() {
            self.render_process_tree(ui, *child_pid, indent + 16);
        }
    }
}

impl eframe::App for ProcessTreeApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        if self.auto_refresh && self.last_update.elapsed().as_secs() >= self.refresh_interval {
            self.tree = tree::build_process_tree();
            self.update_process_cache();
            self.last_update = std::time::Instant::now();
        }

        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading(
                egui::RichText::new("Process Tree Viewer")
                    .color(egui::Color32::from_rgb(70, 130, 180))
                    .strong()
            );

            ui.horizontal(|ui| {
                ui.visuals_mut().widgets.inactive.bg_fill = egui::Color32::from_rgb(70, 130, 180);
                ui.visuals_mut().widgets.hovered.bg_fill = egui::Color32::from_rgb(70, 130, 180);

                if ui.add(
                    egui::RadioButton::new(
                        self.view_mode == ViewMode::Tree,
                        egui::RichText::new("Tree View").color(egui::Color32::from_rgb(70, 130, 180))
                    )
                ).clicked() {
                    self.view_mode = ViewMode::Tree;
                }

                if ui.add(
                    egui::RadioButton::new(
                        self.view_mode == ViewMode::Table,
                        egui::RichText::new("Table View").color(egui::Color32::from_rgb(70, 130, 180))
                    )
                ).clicked() {
                    self.view_mode = ViewMode::Table;
                }

                if ui.button(
                    egui::RichText::new("🔄 Refresh").color(egui::Color32::from_rgb(70, 130, 180))
                ).clicked() {
                    self.tree = tree::build_process_tree();
                    self.update_process_cache();
                    self.last_update = std::time::Instant::now();
                }

                ui.checkbox(&mut self.auto_refresh,
                    egui::RichText::new("Auto-refresh").color(egui::Color32::from_rgb(70, 130, 180))
                );

                if self.auto_refresh {
                    ui.add(
                        egui::Slider::new(&mut self.refresh_interval, 1..=60)
                            .text("Refresh interval (s)")
                    );
                }
            });

            ui.separator();

            egui::ScrollArea::vertical()
                .auto_shrink([false, false])
                .show(ui, |ui| {
                    ui.set_min_size(ui.available_size());
                    match self.view_mode {
                        ViewMode::Tree => self.render_process_tree(ui, 1, 0),
                        ViewMode::Table => self.render_process_table(ui),
                    }
                });

            ui.label(
                egui::RichText::new(format!(
                    "Last updated: {} seconds ago | Total processes: {}",
                    self.last_update.elapsed().as_secs(),
                    self.all_processes.len()
                )).color(egui::Color32::from_rgb(70, 130, 180))
            );
        });

        if self.auto_refresh {
            ctx.request_repaint();
        }
    }
}

fn main() -> eframe::Result<()> {
    let options = eframe::NativeOptions {
        initial_window_size: Some(egui::vec2(1000.0, 800.0)),
        ..Default::default()
    };
    eframe::run_native(
        "Process Tree Viewer",
        options,
        Box::new(|_cc| Box::new(ProcessTreeApp::default())),
    )
}
