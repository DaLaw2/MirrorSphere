use crate::core::infrastructure::app_config::AppConfig;
use crate::core::infrastructure::communication_manager::CommunicationManager;
use crate::model::core::backup::communication::BackupCommand;
use crate::model::core::backup::execution::*;
use crate::model::core::schedule::communication::*;
use crate::model::core::schedule::schedule::*;
use crate::model::error::Error;
use crate::ui::common::{ComparisonModeSelection, FolderSelectionMode};
use eframe::egui;
use egui_file_dialog::FileDialog;
use futures::executor::block_on;
use std::path::PathBuf;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tracing::error;
use uuid::Uuid;

pub struct SchedulePage {
    app_config: Arc<AppConfig>,
    communication_manager: Arc<CommunicationManager>,

    schedules: Vec<Schedule>,

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

    // Edit functionality
    editing_schedule: Option<Schedule>,
    show_edit_schedule_dialog: bool,
    edit_schedule_name: String,
    edit_schedule_source: String,
    edit_schedule_destination: String,
    edit_schedule_interval: ScheduleInterval,
    edit_schedule_mirror: bool,
    edit_schedule_backup_permission: bool,
    edit_schedule_follow_symlinks: bool,
    edit_schedule_comparison_mode: ComparisonModeSelection,
    edit_schedule_hash_type: HashType,

    file_dialog: FileDialog,
    folder_selection_mode: Option<FolderSelectionMode>,

    pub show_disabled_schedules: bool,
    viewing_schedule_details: Option<Uuid>,
    last_refresh: Option<Instant>,
}

impl SchedulePage {
    pub fn new(
        app_config: Arc<AppConfig>,
        communication_manager: Arc<CommunicationManager>,
    ) -> Result<Self, Error> {
        let schedule_page = Self {
            app_config,
            communication_manager,
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

            // Initialize edit fields
            editing_schedule: None,
            show_edit_schedule_dialog: false,
            edit_schedule_name: String::new(),
            edit_schedule_source: String::new(),
            edit_schedule_destination: String::new(),
            edit_schedule_interval: ScheduleInterval::Daily,
            edit_schedule_mirror: false,
            edit_schedule_backup_permission: false,
            edit_schedule_follow_symlinks: false,
            edit_schedule_comparison_mode: ComparisonModeSelection::Standard,
            edit_schedule_hash_type: HashType::BLAKE3,

            file_dialog: FileDialog::new(),
            folder_selection_mode: None,
            show_disabled_schedules: true,
            viewing_schedule_details: None,
            last_refresh: None,
        };
        Ok(schedule_page)
    }

    fn load_schedules(&mut self) {
        match block_on(async {
            self.communication_manager
                .send_query(ScheduleManagerQuery::GetSchedules)
                .await
        }) {
            Ok(ScheduleManagerQueryResponse::GetSchedules(schedules)) => {
                self.schedules = schedules;
            }
            Err(err) => {
                error!("{}", err);
            }
        }
    }

    fn handle_add_schedule(&self, schedule: Schedule) -> Result<(), Error> {
        block_on(async {
            self.communication_manager
                .send_command(ScheduleManagerCommand::AddSchedule(schedule))
                .await?;
            Ok(())
        })
    }

    fn handle_modify_schedule(&self, schedule: Schedule) -> Result<(), Error> {
        block_on(async {
            self.communication_manager
                .send_command(ScheduleManagerCommand::ModifySchedule(schedule))
                .await?;
            Ok(())
        })
    }

    fn handle_remove_schedule(&self, uuid: Uuid) -> Result<(), Error> {
        block_on(async {
            self.communication_manager
                .send_command(ScheduleManagerCommand::RemoveSchedule(uuid))
                .await?;
            Ok(())
        })
    }

    fn handle_active_schedule(&self, uuid: Uuid) -> Result<(), Error> {
        block_on(async {
            self.communication_manager
                .send_command(ScheduleManagerCommand::ActivateSchedule(uuid))
                .await?;
            Ok(())
        })
    }

    fn handle_pause_schedule(&self, uuid: Uuid) -> Result<(), Error> {
        block_on(async {
            self.communication_manager
                .send_command(ScheduleManagerCommand::PauseSchedule(uuid))
                .await?;
            Ok(())
        })
    }

    fn handle_disable_schedule(&self, uuid: Uuid) -> Result<(), Error> {
        block_on(async {
            self.communication_manager
                .send_command(ScheduleManagerCommand::DisableSchedule(uuid))
                .await?;
            Ok(())
        })
    }

    fn handle_run_schedule_now(&self, schedule: Schedule) -> Result<(), Error> {
        block_on(async {
            let execution = schedule.to_execution();
            let uuid = execution.uuid;
            self.communication_manager
                .send_command(BackupCommand::AddExecution(execution))
                .await?;
            self.communication_manager
                .send_command(BackupCommand::StartExecution(uuid))
                .await?;
            Ok(())
        })
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
                    let schedules_to_show: Vec<Schedule> = self
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
        self.draw_edit_schedule_dialog(ctx);
        self.draw_schedule_details_window(ctx);
    }

    fn draw_schedule_item(&mut self, ui: &mut egui::Ui, schedule: &Schedule) {
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
                                    if let Err(err) = self.handle_pause_schedule(schedule.uuid) {
                                        error!("{}", err);
                                    }
                                }
                            }
                            ScheduleState::Paused => {
                                if ui.button("▶ Resume").clicked() {
                                    if let Err(err) = self.handle_active_schedule(schedule.uuid) {
                                        error!("{}", err);
                                    }
                                }
                            }
                            ScheduleState::Disabled => {
                                if ui.button("▶ Enable").clicked() {
                                    if let Err(err) = self.handle_active_schedule(schedule.uuid) {
                                        error!("{}", err);
                                    }
                                }
                            }
                        }

                        if schedule.state != ScheduleState::Disabled
                            && ui.button("❌ Disable").clicked()
                        {
                            if let Err(err) = self.handle_disable_schedule(schedule.uuid) {
                                error!("{}", err);
                            }
                        }

                        if ui.button("🗑").clicked() {
                            if let Err(err) = self.handle_remove_schedule(schedule.uuid) {
                                error!("{}", err);
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
                            ui.add_sized(
                                [300.0, 20.0],
                                egui::TextEdit::singleline(&mut self.new_schedule_name),
                            );
                            ui.label("");
                            ui.end_row();

                            ui.label("Source Path:");
                            ui.add_sized(
                                [300.0, 20.0],
                                egui::TextEdit::singleline(&mut self.new_schedule_source),
                            );
                            if ui.button("📁 Browse").clicked() {
                                self.folder_selection_mode = Some(FolderSelectionMode::Source);
                                self.file_dialog.pick_directory();
                            }
                            ui.end_row();

                            ui.label("Destination Path:");
                            ui.add_sized(
                                [300.0, 20.0],
                                egui::TextEdit::singleline(&mut self.new_schedule_destination),
                            );
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
                        ui.radio_value(
                            &mut self.new_schedule_comparison_mode,
                            ComparisonModeSelection::Standard,
                            "⚡ Standard (Size + Time)",
                        );
                        ui.radio_value(
                            &mut self.new_schedule_comparison_mode,
                            ComparisonModeSelection::Advanced,
                            "🔧 Advanced (+ Attributes)",
                        );
                        ui.radio_value(
                            &mut self.new_schedule_comparison_mode,
                            ComparisonModeSelection::Thorough,
                            "🔍 Thorough (+ Checksum)",
                        );
                    });

                    if self.new_schedule_comparison_mode == ComparisonModeSelection::Thorough {
                        ui.horizontal(|ui| {
                            ui.label("  Hash Algorithm:");
                            egui::ComboBox::from_id_salt("schedule_hash_type")
                                .selected_text(format!("{:?}", self.new_schedule_hash_type))
                                .show_ui(ui, |ui| {
                                    ui.selectable_value(
                                        &mut self.new_schedule_hash_type,
                                        HashType::BLAKE3,
                                        "BLAKE3 (Recommended)",
                                    );
                                    ui.selectable_value(
                                        &mut self.new_schedule_hash_type,
                                        HashType::SHA256,
                                        "SHA256",
                                    );
                                    ui.selectable_value(
                                        &mut self.new_schedule_hash_type,
                                        HashType::SHA3,
                                        "SHA3",
                                    );
                                    ui.selectable_value(
                                        &mut self.new_schedule_hash_type,
                                        HashType::BLAKE2B,
                                        "BLAKE2B",
                                    );
                                    ui.selectable_value(
                                        &mut self.new_schedule_hash_type,
                                        HashType::BLAKE2S,
                                        "BLAKE2S",
                                    );
                                    ui.selectable_value(
                                        &mut self.new_schedule_hash_type,
                                        HashType::MD5,
                                        "MD5 (Legacy)",
                                    );
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
                                ComparisonModeSelection::Thorough => {
                                    Some(ComparisonMode::Thorough(self.new_schedule_hash_type))
                                }
                            };

                            let schedule = Schedule {
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

                            if let Err(err) = self.handle_add_schedule(schedule) {
                                error!("{}", err);
                            }

                            self.reset_schedule_form();
                        }

                        if ui.button("Cancel").clicked() {
                            self.show_add_schedule_dialog = false;
                        }
                    });
                });
        }

        // Handle file dialog for both add and edit modes
        if !self.show_edit_schedule_dialog {
            self.handle_file_dialog_for_add_mode(ctx);
        }
    }

    // New function to draw edit schedule dialog
    fn draw_edit_schedule_dialog(&mut self, ctx: &egui::Context) {
        if self.show_edit_schedule_dialog {
            egui::Window::new("Edit Backup Schedule")
                .collapsible(false)
                .resizable(false)
                .show(ctx, |ui| {
                    egui::Grid::new("edit_schedule_grid")
                        .num_columns(3)
                        .spacing([10.0, 4.0])
                        .show(ui, |ui| {
                            ui.label("Schedule Name:");
                            ui.add_sized(
                                [300.0, 20.0],
                                egui::TextEdit::singleline(&mut self.edit_schedule_name),
                            );
                            ui.label("");
                            ui.end_row();

                            ui.label("Source Path:");
                            ui.add_sized(
                                [300.0, 20.0],
                                egui::TextEdit::singleline(&mut self.edit_schedule_source),
                            );
                            if ui.button("📁 Browse").clicked() {
                                self.folder_selection_mode = Some(FolderSelectionMode::Source);
                                self.file_dialog.pick_directory();
                            }
                            ui.end_row();

                            ui.label("Destination Path:");
                            ui.add_sized(
                                [300.0, 20.0],
                                egui::TextEdit::singleline(&mut self.edit_schedule_destination),
                            );
                            if ui.button("📁 Browse").clicked() {
                                self.folder_selection_mode = Some(FolderSelectionMode::Destination);
                                self.file_dialog.pick_directory();
                            }
                            ui.end_row();

                            ui.label("Interval:");
                            egui::ComboBox::from_label("")
                                .selected_text(format!("{:?}", self.edit_schedule_interval))
                                .show_ui(ui, |ui| {
                                    ui.selectable_value(
                                        &mut self.edit_schedule_interval,
                                        ScheduleInterval::Once,
                                        "Once",
                                    );
                                    ui.selectable_value(
                                        &mut self.edit_schedule_interval,
                                        ScheduleInterval::Daily,
                                        "Daily",
                                    );
                                    ui.selectable_value(
                                        &mut self.edit_schedule_interval,
                                        ScheduleInterval::Weekly,
                                        "Weekly",
                                    );
                                    ui.selectable_value(
                                        &mut self.edit_schedule_interval,
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
                        ui.radio_value(
                            &mut self.edit_schedule_comparison_mode,
                            ComparisonModeSelection::Standard,
                            "⚡ Standard (Size + Time)",
                        );
                        ui.radio_value(
                            &mut self.edit_schedule_comparison_mode,
                            ComparisonModeSelection::Advanced,
                            "🔧 Advanced (+ Attributes)",
                        );
                        ui.radio_value(
                            &mut self.edit_schedule_comparison_mode,
                            ComparisonModeSelection::Thorough,
                            "🔍 Thorough (+ Checksum)",
                        );
                    });

                    if self.edit_schedule_comparison_mode == ComparisonModeSelection::Thorough {
                        ui.horizontal(|ui| {
                            ui.label("  Hash Algorithm:");
                            egui::ComboBox::from_id_salt("edit_schedule_hash_type")
                                .selected_text(format!("{:?}", self.edit_schedule_hash_type))
                                .show_ui(ui, |ui| {
                                    ui.selectable_value(
                                        &mut self.edit_schedule_hash_type,
                                        HashType::BLAKE3,
                                        "BLAKE3 (Recommended)",
                                    );
                                    ui.selectable_value(
                                        &mut self.edit_schedule_hash_type,
                                        HashType::SHA256,
                                        "SHA256",
                                    );
                                    ui.selectable_value(
                                        &mut self.edit_schedule_hash_type,
                                        HashType::SHA3,
                                        "SHA3",
                                    );
                                    ui.selectable_value(
                                        &mut self.edit_schedule_hash_type,
                                        HashType::BLAKE2B,
                                        "BLAKE2B",
                                    );
                                    ui.selectable_value(
                                        &mut self.edit_schedule_hash_type,
                                        HashType::BLAKE2S,
                                        "BLAKE2S",
                                    );
                                    ui.selectable_value(
                                        &mut self.edit_schedule_hash_type,
                                        HashType::MD5,
                                        "MD5 (Legacy)",
                                    );
                                });
                        });
                    }

                    ui.separator();

                    ui.label("Additional Options:");
                    ui.checkbox(&mut self.edit_schedule_follow_symlinks, "Follow Symlinks");
                    ui.checkbox(
                        &mut self.edit_schedule_mirror,
                        "Mirror Mode (Delete extra files in destination)",
                    );
                    ui.checkbox(
                        &mut self.edit_schedule_backup_permission,
                        "Backup File Permissions",
                    );

                    ui.separator();

                    ui.horizontal(|ui| {
                        if ui.button("Update Schedule").clicked()
                            && !self.edit_schedule_name.is_empty()
                            && !self.edit_schedule_source.is_empty()
                            && !self.edit_schedule_destination.is_empty()
                        {
                            if let Some(mut editing_schedule) = self.editing_schedule.clone() {
                                let comparison_mode = match self.edit_schedule_comparison_mode {
                                    ComparisonModeSelection::Standard => {
                                        Some(ComparisonMode::Standard)
                                    }
                                    ComparisonModeSelection::Advanced => {
                                        Some(ComparisonMode::Advanced)
                                    }
                                    ComparisonModeSelection::Thorough => {
                                        Some(ComparisonMode::Thorough(self.edit_schedule_hash_type))
                                    }
                                };

                                editing_schedule.name = self.edit_schedule_name.clone();
                                editing_schedule.source_path =
                                    PathBuf::from(&self.edit_schedule_source);
                                editing_schedule.destination_path =
                                    PathBuf::from(&self.edit_schedule_destination);
                                editing_schedule.interval = self.edit_schedule_interval;
                                editing_schedule.comparison_mode = comparison_mode;
                                editing_schedule.options = BackupOptions {
                                    mirror: self.edit_schedule_mirror,
                                    backup_permission: self.edit_schedule_backup_permission,
                                    follow_symlinks: self.edit_schedule_follow_symlinks,
                                };
                                editing_schedule.updated_at = chrono::Utc::now().naive_utc();

                                if let Err(err) = self.handle_modify_schedule(editing_schedule) {
                                    error!("{}", err);
                                }

                                self.reset_edit_schedule_form();
                            }
                        }

                        if ui.button("Cancel").clicked() {
                            self.reset_edit_schedule_form();
                        }
                    });
                });
        }

        // Handle file dialog for edit mode
        if self.show_edit_schedule_dialog {
            self.handle_file_dialog_for_edit_mode(ctx);
        }
    }

    fn draw_schedule_details_window(&mut self, ctx: &egui::Context) {
        if let Some(schedule_id) = self.viewing_schedule_details {
            let mut show_window = true;
            let mut run_now_clicked = false;
            let mut edit_clicked = false;

            // Clone the schedule data we need before entering the closure
            let schedule_data = self.schedules.iter()
                .find(|s| s.uuid == schedule_id)
                .cloned();

            if let Some(schedule) = schedule_data {
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
                                        }
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
                            // Use local flags to track button clicks
                            if ui.button("▶ Run Now").clicked() {
                                run_now_clicked = true;
                            }

                            if ui.button("✏ Edit").clicked() {
                                edit_clicked = true;
                            }
                        });
                    });

                if run_now_clicked {
                    if let Err(err) = self.handle_run_schedule_now(schedule.clone()) {
                        error!("Failed to run schedule now: {}", err);
                    } else {
                        self.load_schedules();
                    }
                }
                if edit_clicked {
                    self.start_editing_schedule(schedule);
                }
            }
            if !show_window {
                self.viewing_schedule_details = None;
            }
        }
    }

    fn start_editing_schedule(&mut self, schedule: Schedule) {
        self.editing_schedule = Some(schedule.clone());
        self.edit_schedule_name = schedule.name.clone();
        self.edit_schedule_source = schedule.source_path.to_string_lossy().to_string();
        self.edit_schedule_destination = schedule.destination_path.to_string_lossy().to_string();
        self.edit_schedule_interval = schedule.interval;
        self.edit_schedule_mirror = schedule.options.mirror;
        self.edit_schedule_backup_permission = schedule.options.backup_permission;
        self.edit_schedule_follow_symlinks = schedule.options.follow_symlinks;

        if let Some(comparison_mode) = &schedule.comparison_mode {
            match comparison_mode {
                ComparisonMode::Standard => {
                    self.edit_schedule_comparison_mode = ComparisonModeSelection::Standard;
                }
                ComparisonMode::Advanced => {
                    self.edit_schedule_comparison_mode = ComparisonModeSelection::Advanced;
                }
                ComparisonMode::Thorough(hash_type) => {
                    self.edit_schedule_comparison_mode = ComparisonModeSelection::Thorough;
                    self.edit_schedule_hash_type = *hash_type;
                }
            }
        } else {
            self.edit_schedule_comparison_mode = ComparisonModeSelection::Standard;
        }

        self.show_edit_schedule_dialog = true;
        self.viewing_schedule_details = None;
    }

    fn reset_edit_schedule_form(&mut self) {
        self.editing_schedule = None;
        self.edit_schedule_name.clear();
        self.edit_schedule_source.clear();
        self.edit_schedule_destination.clear();
        self.edit_schedule_interval = ScheduleInterval::Daily;
        self.edit_schedule_mirror = false;
        self.edit_schedule_backup_permission = false;
        self.edit_schedule_follow_symlinks = false;
        self.edit_schedule_comparison_mode = ComparisonModeSelection::Standard;
        self.edit_schedule_hash_type = HashType::BLAKE3;
        self.show_edit_schedule_dialog = false;
    }

    fn handle_file_dialog_for_add_mode(&mut self, ctx: &egui::Context) {
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

    fn handle_file_dialog_for_edit_mode(&mut self, ctx: &egui::Context) {
        self.file_dialog.update(ctx);

        if let Some(path) = self.file_dialog.take_picked() {
            if let Some(mode) = &self.folder_selection_mode {
                match mode {
                    FolderSelectionMode::Source => {
                        self.edit_schedule_source = path.to_string_lossy().to_string();
                    }
                    FolderSelectionMode::Destination => {
                        self.edit_schedule_destination = path.to_string_lossy().to_string();
                    }
                }
            }
            self.folder_selection_mode = None;
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
