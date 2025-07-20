use crate::core::backup_engine::BackupEngine;
use crate::core::event_bus::EventBus;
use crate::core::schedule_manager::ScheduleManager;
use crate::ui::execution_page::ExecutionPage;
use crate::ui::schedule_page::SchedulePage;
use crate::model::log::system::SystemLog;
use eframe::egui;
use eframe::{App, Frame};
use macros::log;
use std::sync::Arc;
use crate::core::app_config::AppConfig;

#[derive(Debug, Clone, PartialEq)]
enum PageType {
    Executions,
    Schedules,
}

pub struct MainPage {
    current_page: PageType,
    execution_page: ExecutionPage,
    schedule_page: SchedulePage,
}

impl MainPage {
    pub fn new(
        config: Arc<AppConfig>,
        event_bus: Arc<EventBus>,
        backup_engine: Arc<BackupEngine>,
        schedule_manager: Arc<ScheduleManager>,
    ) -> Self {
        Self {
            current_page: PageType::Executions,
            execution_page: ExecutionPage::new(config.clone(), event_bus, backup_engine),
            schedule_page: SchedulePage::new(config, schedule_manager),
        }
    }

    fn draw_top_panel(&mut self, ctx: &egui::Context) {
        egui::TopBottomPanel::top("top_panel").show(ctx, |ui| {
            egui::menu::bar(ui, |ui| {
                ui.menu_button("File", |ui| {
                    if ui.button("Exit").clicked() {
                        ctx.send_viewport_cmd(egui::ViewportCommand::Close);
                    }
                });

                ui.menu_button("View", |ui| {
                    match self.current_page {
                        PageType::Executions => {
                            ui.checkbox(&mut self.execution_page.show_completed_tasks, "Show Completed Tasks");
                            ui.checkbox(&mut self.execution_page.auto_scroll_errors, "Auto-scroll Error Messages");
                        }
                        PageType::Schedules => {
                            ui.checkbox(&mut self.schedule_page.show_disabled_schedules, "Show Disabled Schedules");
                        }
                    }
                });
            });
        });
    }

    fn draw_tabs(&mut self, ctx: &egui::Context) {
        egui::TopBottomPanel::top("tabs_panel").show(ctx, |ui| {
            ui.horizontal(|ui| {
                ui.selectable_value(&mut self.current_page, PageType::Executions, "📋 Executions");
                ui.selectable_value(&mut self.current_page, PageType::Schedules, "⏰ Schedules");
            });
        });
    }
}

impl App for MainPage {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut Frame) {
        ctx.request_repaint_after(std::time::Duration::from_millis(100));

        self.draw_top_panel(ctx);
        self.draw_tabs(ctx);

        match self.current_page {
            PageType::Executions => self.execution_page.update(ctx),
            PageType::Schedules => self.schedule_page.update(ctx),
        }
    }

    fn on_exit(&mut self, _gl: Option<&eframe::glow::Context>) {
        log!(SystemLog::GuiExited)
    }
}
