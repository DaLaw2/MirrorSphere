use crate::core::app_config::AppConfig;
use crate::core::schedule_manager::ScheduleManager;
use crate::model::backup::backup_execution::*;
use crate::model::backup::backup_schedule::*;
use crate::model::error::task::TaskError;
use crate::ui::common::{ComparisonModeSelection, FolderSelectionMode};
use eframe::egui;
use egui_file_dialog::FileDialog;
use futures::executor::block_on;
use macros::log;
use std::path::PathBuf;
use std::sync::Arc;
use std::time::{Duration, Instant};
use uuid::Uuid;

pub struct SchedulePage {
    app_config: Arc<AppConfig>,
    schedule_manager: Arc<ScheduleManager>,

    schedules: Vec<BackupSchedule>,

    new_schedule_name: String,
    new_schedule_source: String,
    new_schedule_destination: String,
    new_schedule_interval: ScheduleInterval,
    new_schedule_mirror: bool,
    new_schedule_backup_permission: bool,
    new_schedule_follow_symlinks: bool,
    new_schedule_comparison_mode: ComparisonModeSelection,
    new_schedule_hash_type: HashType,
    show_add_schedule_dialog: bool,

    file_dialog: FileDialog,
    folder_selection_mode: Option<FolderSelectionMode>,

    pub show_disabled_schedules: bool,
    viewing_schedule_details: Option<Uuid>,
    last_refresh: Option<Instant>,
}

impl SchedulePage {
    pub fn new(app_config: Arc<AppConfig>, schedule_manager: Arc<ScheduleManager>) -> Self {
        Self {
            app_config,
            schedule_manager,
            schedules: Vec::new(),
            new_schedule_name: String::new(),
            new_schedule_source: String::new(),
            new_schedule_destination: String::new(),
            new_schedule_interval: ScheduleInterval::Daily,
            new_schedule_mirror: false,
            new_schedule_backup_permission: false,
            new_schedule_follow_symlinks: false,
            new_schedule_comparison_mode: ComparisonModeSelection::Standard,
            new_schedule_hash_type: HashType::BLAKE3,
            show_add_schedule_dialog: false,
            file_dialog: FileDialog::new(),
            folder_selection_mode: None,
            show_disabled_schedules: true,
            viewing_schedule_details: None,
            last_refresh: None,
        }
    }

    fn load_schedules(&mut self) {
        match block_on(self.schedule_manager.get_all_schedules()) {
            Ok(schedules) => self.schedules = schedules,
            Err(err) => log!(TaskError::LoadScheduleFailed(err)),
        }
    }

    pub fn update(&mut self, ctx: &egui::Context) {
        let should_refresh = match self.last_refresh {
            None => true,
            Some(last) => {
                last.elapsed() > Duration::from_secs(self.app_config.ui_refresh_time as u64)
            }
        };

        if should_refresh {
            self.load_schedules();
            self.last_refresh = Some(Instant::now());
        }

        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("Backup Schedules");

            ui.horizontal(|ui| {
                if ui.button("🔄 Refresh").clicked() {
                    self.load_schedules();
                    self.last_refresh = Some(Instant::now());
                }

                if ui.button("➕ Add Schedule").clicked() {
                    self.show_add_schedule_dialog = true;
                }

                ui.separator();

                let active_count = self
                    .schedules
                    .iter()
                    .filter(|s| s.state == ScheduleState::Active)
                    .count();
                ui.label(format!("Active: {active_count}"));

                let paused_count = self
                    .schedules
                    .iter()
                    .filter(|s| s.state == ScheduleState::Paused)
                    .count();
                ui.label(format!("Paused: {paused_count}"));

                let disabled_count = self
                    .schedules
                    .iter()
                    .filter(|s| s.state == ScheduleState::Disabled)
                    .count();
                ui.label(format!("Disabled: {disabled_count}"));
            });

            ui.separator();

            egui::ScrollArea::vertical()
                .auto_shrink([false; 2])
                .show(ui, |ui| {
                    let schedules_to_show: Vec<BackupSchedule> = self
                        .schedules
                        .iter()
                        .filter(|schedule| {
                            self.show_disabled_schedules
                                || schedule.state != ScheduleState::Disabled
                        })
                        .cloned()
                        .collect();

                    for schedule in schedules_to_show {
                        self.draw_schedule_item(ui, &schedule);
                        ui.separator();
                    }

                    if self.schedules.is_empty() {
                        ui.vertical_centered(|ui| {
                            ui.label("⏰ No backup schedules");
                            ui.label("Click the button above to add a schedule");
                        });
                    }
                });
        });

        self.draw_add_schedule_dialog(ctx);
        self.draw_schedule_details_window(ctx);
    }

    fn draw_schedule_item(&mut self, ui: &mut egui::Ui, schedule: &BackupSchedule) {
        egui::Frame::new()
            .fill(ui.visuals().faint_bg_color)
            .inner_margin(8.0)
            .show(ui, |ui| {
                ui.horizontal(|ui| {
                    ui.vertical(|ui| {
                        ui.label(format!("📅 {}", schedule.name));
                        ui.label(format!("🗂️ {}", schedule.source_path.display()));
                        ui.label(format!("📁 {}", schedule.destination_path.display()));
                        ui.label(format!("⏱ {:?}", schedule.interval));

                        ui.horizontal(|ui| {
                            let (color, symbol, status_text) = match schedule.state {
                                ScheduleState::Active => (egui::Color32::GREEN, "✅", "Active"),
                                ScheduleState::Paused => (egui::Color32::YELLOW, "⏸", "Paused"),
                                ScheduleState::Disabled => (egui::Color32::GRAY, "❌", "Disabled"),
                            };

                            ui.colored_label(color, format!("{symbol} {status_text}"));

                            if let Some(comparison_mode) = &schedule.comparison_mode {
                                ui.separator();
                                let mode_text = match comparison_mode {
                                    ComparisonMode::Standard => "⚡ Standard",
                                    ComparisonMode::Advanced => "🔧 Advanced",
                                    ComparisonMode::Thorough(_) => "🔍 Thorough",
                                };
                                ui.label(mode_text);
                            }

                            if let Some(last_run) = schedule.last_run_time {
                                ui.separator();
                                ui.label(format!(
                                    "Last run: {}",
                                    last_run.format("%Y-%m-%d %H:%M")
                                ));
                            }

                            if let Some(next_run) = schedule.next_run_time {
                                ui.separator();
                                ui.label(format!(
                                    "Next run: {}",
                                    next_run.format("%Y-%m-%d %H:%M")
                                ));
                            }
                        });
                    });

                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        if ui.small_button("👁 Details").clicked() {
                            self.viewing_schedule_details = Some(schedule.uuid);
                        }

                        ui.separator();

                        match schedule.state {
                            ScheduleState::Active => {
                                if ui.button("⏸ Pause").clicked() {
                                    if let Err(err) = block_on(
                                        self.schedule_manager.pause_schedule(schedule.uuid),
                                    ) {
                                        log!(TaskError::PauseScheduleFailed(err));
                                    }
                                }
                            }
                            ScheduleState::Paused => {
                                if ui.button("▶ Resume").clicked() {
                                    if let Err(err) = block_on(
                                        self.schedule_manager.active_schedule(schedule.uuid),
                                    ) {
                                        log!(TaskError::EnableScheduleFailed(err));
                                    }
                                }
                            }
                            ScheduleState::Disabled => {
                                if ui.button("▶ Enable").clicked() {
                                    if let Err(err) = block_on(
                                        self.schedule_manager.active_schedule(schedule.uuid),
                                    ) {
                                        log!(TaskError::EnableScheduleFailed(err));
                                    }
                                }
                            }
                        }

                        if schedule.state != ScheduleState::Disabled
                            && ui.button("❌ Disable").clicked()
                        {
                            if let Err(err) =
                                block_on(self.schedule_manager.disable_schedule(schedule.uuid))
                            {
                                log!(TaskError::DisableScheduleFailed(err));
                            }
                        }

                        if ui.button("🗑").clicked() {
                            if let Err(err) =
                                block_on(self.schedule_manager.remove_schedule(schedule.uuid))
                            {
                                log!(TaskError::RemoveScheduleFailed(err));
                            }
                        }
                    });
                });
            });
    }

    fn draw_add_schedule_dialog(&mut self, ctx: &egui::Context) {
        if self.show_add_schedule_dialog {
            egui::Window::new("Add Backup Schedule")
                .collapsible(false)
                .resizable(false)
                .show(ctx, |ui| {
                    egui::Grid::new("add_schedule_grid")
                        .num_columns(3)
                        .spacing([10.0, 4.0])
                        .show(ui, |ui| {
                            ui.label("Schedule Name:");
                            ui.add_sized([300.0, 20.0], egui::TextEdit::singleline(&mut self.new_schedule_name));
                            ui.label("");
                            ui.end_row();

                            ui.label("Source Path:");
                            ui.add_sized([300.0, 20.0], egui::TextEdit::singleline(&mut self.new_schedule_source));
                            if ui.button("📁 Browse").clicked() {
                                self.folder_selection_mode = Some(FolderSelectionMode::Source);
                                self.file_dialog.pick_directory();
                            }
                            ui.end_row();

                            ui.label("Destination Path:");
                            ui.add_sized([300.0, 20.0], egui::TextEdit::singleline(&mut self.new_schedule_destination));
                            if ui.button("📁 Browse").clicked() {
                                self.folder_selection_mode = Some(FolderSelectionMode::Destination);
                                self.file_dialog.pick_directory();
                            }
                            ui.end_row();

                            ui.label("Interval:");
                            egui::ComboBox::from_label("")
                                .selected_text(format!("{:?}", self.new_schedule_interval))
                                .show_ui(ui, |ui| {
                                    ui.selectable_value(
                                        &mut self.new_schedule_interval,
                                        ScheduleInterval::Once,
                                        "Once",
                                    );
                                    ui.selectable_value(
                                        &mut self.new_schedule_interval,
                                        ScheduleInterval::Daily,
                                        "Daily",
                                    );
                                    ui.selectable_value(
                                        &mut self.new_schedule_interval,
                                        ScheduleInterval::Weekly,
                                        "Weekly",
                                    );
                                    ui.selectable_value(
                                        &mut self.new_schedule_interval,
                                        ScheduleInterval::Monthly,
                                        "Monthly",
                                    );
                                });
                            ui.label("");
                            ui.end_row();

                        });

                    ui.separator();

                    ui.label("File Comparison Mode:");
                    ui.horizontal(|ui| {
                        ui.radio_value(&mut self.new_schedule_comparison_mode, ComparisonModeSelection::Standard, "⚡ Standard (Size + Time)");
                        ui.radio_value(&mut self.new_schedule_comparison_mode, ComparisonModeSelection::Advanced, "🔧 Advanced (+ Attributes)");
                        ui.radio_value(&mut self.new_schedule_comparison_mode, ComparisonModeSelection::Thorough, "🔍 Thorough (+ Checksum)");
                    });

                    if self.new_schedule_comparison_mode == ComparisonModeSelection::Thorough {
                        ui.horizontal(|ui| {
                            ui.label("  Hash Algorithm:");
                            egui::ComboBox::from_id_salt("schedule_hash_type")
                                .selected_text(format!("{:?}", self.new_schedule_hash_type))
                                .show_ui(ui, |ui| {
                                    ui.selectable_value(&mut self.new_schedule_hash_type, HashType::BLAKE3, "BLAKE3 (Recommended)");
                                    ui.selectable_value(&mut self.new_schedule_hash_type, HashType::SHA256, "SHA256");
                                    ui.selectable_value(&mut self.new_schedule_hash_type, HashType::SHA3, "SHA3");
                                    ui.selectable_value(&mut self.new_schedule_hash_type, HashType::BLAKE2B, "BLAKE2B");
                                    ui.selectable_value(&mut self.new_schedule_hash_type, HashType::BLAKE2S, "BLAKE2S");
                                    ui.selectable_value(&mut self.new_schedule_hash_type, HashType::MD5, "MD5 (Legacy)");
                                });
                        });
                    }

                    ui.separator();

                    ui.label("Additional Options:");
                    ui.checkbox(&mut self.new_schedule_follow_symlinks, "Follow Symlinks");
                    ui.checkbox(
                        &mut self.new_schedule_mirror,
                        "Mirror Mode (Delete extra files in destination)",
                    );
                    ui.checkbox(
                        &mut self.new_schedule_backup_permission,
                        "Backup File Permissions",
                    );

                    ui.separator();

                    ui.horizontal(|ui| {
                        if ui.button("Create Schedule").clicked()
                            && !self.new_schedule_name.is_empty()
                            && !self.new_schedule_source.is_empty()
                            && !self.new_schedule_destination.is_empty()
                        {
                            let comparison_mode = match self.new_schedule_comparison_mode {
                                ComparisonModeSelection::Standard => Some(ComparisonMode::Standard),
                                ComparisonModeSelection::Advanced => Some(ComparisonMode::Advanced),
                                ComparisonModeSelection::Thorough => Some(ComparisonMode::Thorough(self.new_schedule_hash_type)),
                            };

                            let schedule = BackupSchedule {
                                uuid: Uuid::new_v4(),
                                name: self.new_schedule_name.clone(),
                                state: ScheduleState::Active,
                                source_path: PathBuf::from(&self.new_schedule_source),
                                destination_path: PathBuf::from(&self.new_schedule_destination),
                                backup_type: BackupType::Full,
                                comparison_mode,
                                options: BackupOptions {
                                    mirror: self.new_schedule_mirror,
                                    backup_permission: self.new_schedule_backup_permission,
                                    follow_symlinks: self.new_schedule_follow_symlinks,
                                },
                                interval: self.new_schedule_interval,
                                last_run_time: None,
                                next_run_time: None,
                                created_at: chrono::Utc::now().naive_utc(),
                                updated_at: chrono::Utc::now().naive_utc(),
                            };

                            if let Err(e) =
                                block_on(self.schedule_manager.create_schedule(schedule))
                            {
                                eprintln!("Failed to create schedule: {e:?}");
                            }

                            self.reset_schedule_form();
                        }

                        if ui.button("Cancel").clicked() {
                            self.show_add_schedule_dialog = false;
                        }
                    });
                });
        }

        self.file_dialog.update(ctx);

        if let Some(path) = self.file_dialog.take_picked() {
            if let Some(mode) = &self.folder_selection_mode {
                match mode {
                    FolderSelectionMode::Source => {
                        self.new_schedule_source = path.to_string_lossy().to_string();
                    }
                    FolderSelectionMode::Destination => {
                        self.new_schedule_destination = path.to_string_lossy().to_string();
                    }
                }
            }
            self.folder_selection_mode = None;
        }
    }

    fn draw_schedule_details_window(&mut self, ctx: &egui::Context) {
        if let Some(schedule_id) = self.viewing_schedule_details {
            let mut show_window = true;

            if let Some(schedule) = self.schedules.iter().find(|s| s.uuid == schedule_id) {
                egui::Window::new(format!("Schedule Details - {}", schedule.name))
                    .open(&mut show_window)
                    .resizable(true)
                    .default_width(500.0)
                    .default_height(350.0)
                    .show(ctx, |ui| {
                        egui::Grid::new("schedule_details_grid")
                            .num_columns(2)
                            .spacing([10.0, 4.0])
                            .show(ui, |ui| {
                                ui.label("Name:");
                                ui.label(&schedule.name);
                                ui.end_row();

                                ui.label("State:");
                                ui.label(format!("{:?}", schedule.state));
                                ui.end_row();

                                ui.label("Source:");
                                ui.label(schedule.source_path.display().to_string());
                                ui.end_row();

                                ui.label("Destination:");
                                ui.label(schedule.destination_path.display().to_string());
                                ui.end_row();

                                ui.label("Backup Type:");
                                ui.label(format!("{:?}", schedule.backup_type));
                                ui.end_row();

                                if let Some(comparison_mode) = &schedule.comparison_mode {
                                    ui.label("Comparison Mode:");
                                    let mode_text = match comparison_mode {
                                        ComparisonMode::Standard => "Standard (Size + Time)",
                                        ComparisonMode::Advanced => "Advanced (+ Attributes)",
                                        ComparisonMode::Thorough(hash_type) => {
                                            &format!("Thorough (+ Checksum: {hash_type:?})")
                                        },
                                    };
                                    ui.label(mode_text);
                                    ui.end_row();
                                }

                                ui.label("Interval:");
                                ui.label(format!("{:?}", schedule.interval));
                                ui.end_row();

                                if let Some(last_run) = schedule.last_run_time {
                                    ui.label("Last Run:");
                                    ui.label(last_run.format("%Y-%m-%d %H:%M:%S").to_string());
                                    ui.end_row();
                                }

                                if let Some(next_run) = schedule.next_run_time {
                                    ui.label("Next Run:");
                                    ui.label(next_run.format("%Y-%m-%d %H:%M:%S").to_string());
                                    ui.end_row();
                                }

                                ui.label("Created:");
                                ui.label(
                                    schedule.created_at.format("%Y-%m-%d %H:%M:%S").to_string(),
                                );
                                ui.end_row();

                                ui.label("Updated:");
                                ui.label(
                                    schedule.updated_at.format("%Y-%m-%d %H:%M:%S").to_string(),
                                );
                                ui.end_row();
                            });

                        ui.separator();

                        ui.label("Options:");
                        ui.horizontal_wrapped(|ui| {
                            if schedule.options.mirror {
                                ui.label("✅ Mirror Mode");
                            }
                            if schedule.options.backup_permission {
                                ui.label("✅ Backup Permissions");
                            }
                            if schedule.options.follow_symlinks {
                                ui.label("✅ Follow Symlinks");
                            }
                        });

                        ui.separator();

                        ui.horizontal(|ui| {
                            if ui.button("Run Now").clicked() {
                                println!("Would run schedule {} now", schedule.name);
                            }
                            if ui.button("Edit").clicked() {
                                println!("Would edit schedule {}", schedule.name);
                            }
                        });
                    });
            }

            if !show_window {
                self.viewing_schedule_details = None;
            }
        }
    }

    fn reset_schedule_form(&mut self) {
        self.new_schedule_name.clear();
        self.new_schedule_source.clear();
        self.new_schedule_destination.clear();
        self.new_schedule_interval = ScheduleInterval::Daily;
        self.new_schedule_mirror = false;
        self.new_schedule_backup_permission = false;
        self.new_schedule_follow_symlinks = false;
        self.new_schedule_comparison_mode = ComparisonModeSelection::Standard;
        self.new_schedule_hash_type = HashType::BLAKE3;
        self.show_add_schedule_dialog = false;
    }
}
