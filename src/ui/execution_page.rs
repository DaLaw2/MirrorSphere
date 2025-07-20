use crate::core::backup_engine::BackupEngine;
use crate::core::event_bus::EventBus;
use crate::model::backup::backup_execution::*;
use crate::model::error::Error;
use crate::model::event::error::BackupError;
use crate::model::event::execution::*;
use crate::model::event::filesystem::FolderProcessing;
use dashmap::DashMap;
use eframe::egui;
use egui_file_dialog::FileDialog;
use futures::executor::block_on;
use std::path::PathBuf;
use std::sync::Arc;
use std::sync::mpsc::Receiver;
use uuid::Uuid;
use crate::core::app_config::AppConfig;

#[derive(Debug, Clone)]
struct ExecutionDisplay {
    execution: BackupExecution,
    current_folder: String,
    processed_files: usize,
    error_count: usize,
}

impl From<BackupExecution> for ExecutionDisplay {
    fn from(execution: BackupExecution) -> Self {
        Self {
            execution,
            current_folder: String::new(),
            processed_files: 0,
            error_count: 0,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
enum FolderSelectionMode {
    Source,
    Destination,
}

pub struct ExecutionPage {
    config: Arc<AppConfig>,
    backup_engine: Arc<BackupEngine>,

    folder_processing_events: Receiver<FolderProcessing>,
    progress_events: Receiver<ExecutionProgress>,
    backup_error_events: Receiver<BackupError>,

    executions: DashMap<Uuid, ExecutionDisplay>,
    error_messages: DashMap<Uuid, Vec<Error>>,

    new_task_source: String,
    new_task_destination: String,
    new_task_mirror: bool,
    new_task_lock_source: bool,
    new_task_backup_permission: bool,
    new_task_follow_symlinks: bool,
    show_add_task_dialog: bool,

    file_dialog: FileDialog,
    folder_selection_mode: Option<FolderSelectionMode>,

    pub auto_scroll_errors: bool,
    pub show_completed_tasks: bool,
    viewing_errors_for_task: Option<Uuid>,
}

impl ExecutionPage {
    pub fn new(config: Arc<AppConfig>, event_bus: Arc<EventBus>, backup_engine: Arc<BackupEngine>) -> Self {
        let folder_processing_events = event_bus.subscribe::<FolderProcessing>();
        let progress_events = event_bus.subscribe::<ExecutionProgress>();
        let backup_error_events = event_bus.subscribe::<BackupError>();

        Self {
            config,
            backup_engine,
            folder_processing_events,
            progress_events,
            backup_error_events,
            executions: DashMap::new(),
            error_messages: DashMap::new(),
            new_task_source: String::new(),
            new_task_destination: String::new(),
            new_task_mirror: false,
            new_task_lock_source: false,
            new_task_backup_permission: false,
            new_task_follow_symlinks: false,
            show_add_task_dialog: false,
            file_dialog: FileDialog::new(),
            folder_selection_mode: None,
            auto_scroll_errors: true,
            show_completed_tasks: true,
            viewing_errors_for_task: None,
        }
    }

    fn process_events(&mut self) {
        while let Ok(event) = self.folder_processing_events.try_recv() {
            if let Some(mut task_display) = self.executions.get_mut(&event.execution_id) {
                task_display.current_folder = event.current_folder.to_string_lossy().to_string();
            }
        }

        while let Ok(event) = self.progress_events.try_recv() {
            if let Some(mut task_display) = self.executions.get_mut(&event.task_id) {
                task_display.processed_files = event.processed_files;
                task_display.error_count = event.error_count;
            }
        }

        while let Ok(event) = self.backup_error_events.try_recv() {
            match self.error_messages.get_mut(&event.task_id) {
                Some(mut errors) => errors.push(event.error),
                None => {
                    self.error_messages.insert(event.task_id, vec![event.error]);
                }
            }
        }
    }

    pub fn update(&mut self, ctx: &egui::Context) {
        self.process_events();

        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("Backup Executions");

            ui.horizontal(|ui| {
                if ui.button("➕ Add Execution").clicked() {
                    self.show_add_task_dialog = true;
                }

                ui.separator();

                let running_count = self
                    .executions
                    .iter()
                    .filter(|entry| entry.value().execution.state == BackupState::Running)
                    .count();
                ui.label(format!("Running: {}", running_count));

                let completed_count = self
                    .executions
                    .iter()
                    .filter(|entry| entry.value().execution.state == BackupState::Completed)
                    .count();
                ui.label(format!("Completed: {}", completed_count));

                let error_count: usize = self
                    .error_messages
                    .iter()
                    .map(|entry| entry.value().len())
                    .sum();
                if error_count > 0 {
                    ui.separator();
                    ui.colored_label(egui::Color32::RED, format!("Total Errors: {}", error_count));
                }
            });

            ui.separator();

            egui::ScrollArea::vertical()
                .auto_shrink([false; 2])
                .show(ui, |ui| {
                    let tasks_to_show: Vec<(Uuid, ExecutionDisplay)> = self
                        .executions
                        .iter()
                        .filter_map(|entry| {
                            let (task_id, task_display) = (entry.key(), entry.value());

                            if !self.show_completed_tasks
                                && task_display.execution.state == BackupState::Completed
                            {
                                return None;
                            }

                            Some((*task_id, task_display.clone()))
                        })
                        .collect();

                    for (task_id, task_display) in tasks_to_show {
                        self.draw_execution_item(ui, task_id, &task_display);
                        ui.separator();
                    }

                    if self.executions.is_empty() {
                        ui.vertical_centered(|ui| {
                            ui.label("🚀 No backup executions");
                            ui.label("Click the button above to add an execution");
                        });
                    }
                });
        });

        self.draw_add_execution_dialog(ctx);
        self.draw_execution_errors_window(ctx);
    }

    fn draw_execution_item(&mut self, ui: &mut egui::Ui, task_id: Uuid, task_display: &ExecutionDisplay) {
        egui::Frame::new()
            .fill(ui.visuals().faint_bg_color)
            .inner_margin(8.0)
            .show(ui, |ui| {
                ui.horizontal(|ui| {
                    ui.vertical(|ui| {
                        ui.label(format!("🗂️ {}", task_display.execution.source_path.display()));
                        ui.label(format!("📁 {}", task_display.execution.destination_path.display()));

                        ui.horizontal(|ui| {
                            let (color, symbol) = match task_display.execution.state {
                                BackupState::Running => (egui::Color32::GREEN, "▶️"),
                                BackupState::Suspended => (egui::Color32::YELLOW, "⏸️"),
                                BackupState::Completed => (egui::Color32::BLUE, "✅"),
                                BackupState::Failed => (egui::Color32::RED, "❌"),
                                BackupState::Canceled => (egui::Color32::GRAY, "⏹️"),
                                BackupState::Pending => (egui::Color32::GRAY, "⏸️"),
                            };

                            ui.colored_label(color, format!("{} {:?}", symbol, task_display.execution.state));

                            if !task_display.current_folder.is_empty() {
                                ui.separator();
                                ui.label(format!("📄 {}",
                                                 task_display.current_folder.chars().take(50).collect::<String>()));
                            }
                        });

                        ui.horizontal(|ui| {
                            if task_display.processed_files > 0 || task_display.error_count > 0 {
                                ui.label(format!("📊 Processed: {} | Errors: {}",
                                                 task_display.processed_files, task_display.error_count));
                            }
                        });
                    });

                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        if let Some(errors) = self.error_messages.get(&task_id) {
                            if !errors.is_empty() {
                                if ui.small_button("👁 View Errors").clicked() {
                                    self.viewing_errors_for_task = Some(task_id);
                                }
                                ui.separator();
                            }
                        }

                        match task_display.execution.state {
                            BackupState::Pending | BackupState::Suspended => {
                                if ui.button("▶️ Start").clicked() {
                                    if let Err(e) = block_on(self.backup_engine.start_execution(task_id)) {
                                        eprintln!("Failed to start execution: {:?}", e);
                                    }
                                }
                            }
                            BackupState::Running => {
                                if ui.button("⏸️ Pause").clicked() {
                                    if let Err(e) = block_on(self.backup_engine.suspend_execution(task_id)) {
                                        eprintln!("Failed to suspend execution: {:?}", e);
                                    }
                                }
                            }
                            _ => {}
                        }

                        if task_display.execution.state == BackupState::Suspended {
                            if ui.button("▶️ Resume").clicked() {
                                let rt = tokio::runtime::Handle::current();
                                if let Err(e) = rt.block_on(self.backup_engine.resume_execution(task_id)) {
                                    eprintln!("Failed to resume execution: {:?}", e);
                                }
                            }
                        }

                        if ui.button("🗑️").clicked() {
                            let rt = tokio::runtime::Handle::current();
                            rt.block_on(self.backup_engine.remove_execution(&task_id));
                            self.executions.remove(&task_id);
                            self.error_messages.remove(&task_id);
                            if self.viewing_errors_for_task == Some(task_id) {
                                self.viewing_errors_for_task = None;
                            }
                        }
                    });
                });
            });
    }

    fn draw_add_execution_dialog(&mut self, ctx: &egui::Context) {
        if self.show_add_task_dialog {
            egui::Window::new("Add Backup Execution")
                .collapsible(false)
                .resizable(false)
                .show(ctx, |ui| {
                    egui::Grid::new("add_execution_grid")
                        .num_columns(3)
                        .spacing([10.0, 4.0])
                        .show(ui, |ui| {
                            ui.label("Source Path:");
                            ui.text_edit_singleline(&mut self.new_task_source);
                            if ui.button("📁 Browse").clicked() {
                                self.folder_selection_mode = Some(FolderSelectionMode::Source);
                                self.file_dialog.pick_directory();
                            }
                            ui.end_row();

                            ui.label("Destination Path:");
                            ui.text_edit_singleline(&mut self.new_task_destination);
                            if ui.button("📁 Browse").clicked() {
                                self.folder_selection_mode = Some(FolderSelectionMode::Destination);
                                self.file_dialog.pick_directory();
                            }
                            ui.end_row();
                        });

                    ui.separator();

                    ui.label("Options:");
                    ui.checkbox(&mut self.new_task_follow_symlinks, "Follow Symlinks");
                    ui.checkbox(&mut self.new_task_mirror, "Mirror Mode (Delete extra files in destination)");
                    ui.checkbox(&mut self.new_task_lock_source, "Lock Source Files");
                    ui.checkbox(&mut self.new_task_backup_permission, "Backup File Permissions");

                    ui.separator();

                    ui.horizontal(|ui| {
                        if ui.button("Create Execution").clicked() {
                            if !self.new_task_source.is_empty() && !self.new_task_destination.is_empty() {
                                let execution = BackupExecution {
                                    uuid: Uuid::new_v4(),
                                    state: BackupState::Pending,
                                    source_path: PathBuf::from(&self.new_task_source),
                                    destination_path: PathBuf::from(&self.new_task_destination),
                                    backup_type: BackupType::Full,
                                    comparison_mode: None,
                                    options: BackupOptions {
                                        mirror: self.new_task_mirror,
                                        lock_source: self.new_task_lock_source,
                                        backup_permission: self.new_task_backup_permission,
                                        follow_symlinks: self.new_task_follow_symlinks,
                                    },
                                };

                                let execution_display = ExecutionDisplay::from(execution.clone());
                                self.executions.insert(execution.uuid, execution_display);

                                let rt = tokio::runtime::Handle::current();
                                rt.block_on(self.backup_engine.add_execution(execution));
                                self.reset_form();
                            }
                        }

                        if ui.button("Cancel").clicked() {
                            self.show_add_task_dialog = false;
                        }
                    });
                });
        }

        self.file_dialog.update(ctx);

        if let Some(path) = self.file_dialog.take_picked() {
            if let Some(mode) = &self.folder_selection_mode {
                match mode {
                    FolderSelectionMode::Source => {
                        self.new_task_source = path.to_string_lossy().to_string();
                    }
                    FolderSelectionMode::Destination => {
                        self.new_task_destination = path.to_string_lossy().to_string();
                    }
                }
            }
            self.folder_selection_mode = None;
        }
    }

    fn draw_execution_errors_window(&mut self, ctx: &egui::Context) {
        if let Some(task_id) = self.viewing_errors_for_task {
            let mut show_window = true;

            let window_title = if let Some(task) = self.executions.get(&task_id) {
                format!("Execution Errors - {}",
                        task.execution.source_path.file_name().unwrap_or_default().to_string_lossy())
            } else {
                "Execution Errors".to_string()
            };

            egui::Window::new(window_title)
                .open(&mut show_window)
                .resizable(true)
                .default_width(600.0)
                .default_height(400.0)
                .show(ctx, |ui| {
                    if let Some(errors) = self.error_messages.get(&task_id) {
                        ui.horizontal(|ui| {
                            ui.heading(format!("Error List ({} items)", errors.len()));

                            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                                if ui.button("🗑️ Clear All Errors").clicked() {
                                    self.error_messages.remove(&task_id);
                                    self.viewing_errors_for_task = None;
                                }
                            });
                        });

                        ui.separator();

                        egui::ScrollArea::vertical()
                            .stick_to_bottom(self.auto_scroll_errors)
                            .show(ui, |ui| {
                                for (i, error) in errors.iter().enumerate() {
                                    egui::Frame::new()
                                        .fill(if i % 2 == 0 {
                                            ui.visuals().faint_bg_color
                                        } else {
                                            egui::Color32::TRANSPARENT
                                        })
                                        .inner_margin(4.0)
                                        .show(ui, |ui| {
                                            ui.horizontal(|ui| {
                                                ui.label(format!("{}.", i + 1));
                                                ui.colored_label(egui::Color32::LIGHT_RED, format!("{}", error));
                                            });
                                        });
                                }

                                if errors.is_empty() {
                                    ui.vertical_centered(|ui| {
                                        ui.label("✅ No errors for this execution");
                                    });
                                }
                            });
                    } else {
                        ui.vertical_centered(|ui| {
                            ui.label("⚠️ Cannot find error information for this execution");
                        });
                    }
                });

            if !show_window {
                self.viewing_errors_for_task = None;
            }
        }
    }

    fn reset_form(&mut self) {
        self.new_task_source.clear();
        self.new_task_destination.clear();
        self.new_task_mirror = false;
        self.new_task_lock_source = false;
        self.new_task_backup_permission = false;
        self.new_task_follow_symlinks = false;
        self.show_add_task_dialog = false;
    }
}
