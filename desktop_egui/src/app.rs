use crate::state::AppState;
use crate::ui::{
    chart_panel, data_grid, dialogs, ingest_dialog, menu_bar, sql_panel, stats_panel,
    status_bar, steps_panel, table_list, theme, welcome,
};

pub struct RustoraApp {
    state: AppState,
    theme_applied: bool,
}

impl RustoraApp {
    pub fn new(_cc: &eframe::CreationContext<'_>) -> Self {
        Self {
            state: AppState::new(),
            theme_applied: false,
        }
    }
}

impl eframe::App for RustoraApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        if !self.theme_applied {
            theme::apply_rustora_theme(ctx);
            self.theme_applied = true;
        }

        // Menu bar / toolbar at top
        egui::TopBottomPanel::top("toolbar").show(ctx, |ui| {
            menu_bar::show_menu_bar(ui, &mut self.state);
        });

        // Status bar at bottom
        egui::TopBottomPanel::bottom("statusbar")
            .max_height(24.0)
            .show(ctx, |ui| {
                status_bar::show_status_bar(ui, &mut self.state);
            });

        // SQL panel at bottom (above status bar)
        if self.state.sql_visible {
            egui::TopBottomPanel::bottom("sql_panel")
                .resizable(true)
                .default_height(100.0)
                .max_height(300.0)
                .show(ctx, |ui| {
                    sql_panel::show_sql_panel(ui, &mut self.state);
                });
        }

        // Table list sidebar (left)
        if !self.state.tables.is_empty() {
            egui::SidePanel::left("table_list")
                .resizable(true)
                .default_width(160.0)
                .min_width(120.0)
                .max_width(300.0)
                .show(ctx, |ui| {
                    table_list::show_table_list(ui, &mut self.state);
                });
        }

        // Applied Steps panel (right) -- Phase 1
        if self.state.features.steps_panel
            && self.state.steps_visible
            && !self.state.steps_entries.is_empty()
        {
            egui::SidePanel::right("steps_panel")
                .resizable(true)
                .default_width(200.0)
                .min_width(150.0)
                .max_width(350.0)
                .show(ctx, |ui| {
                    steps_panel::show_steps_panel(ui, &mut self.state);
                });
        }

        // Central panel - welcome or data grid
        egui::CentralPanel::default().show(ctx, |ui| {
            if self.state.columns.is_empty() {
                welcome::show_welcome(ui, &mut self.state);
            } else {
                data_grid::show_data_grid(ui, &mut self.state);
            }
        });

        // Floating windows (dialogs, chart, stats, ingest)
        dialogs::show_dialogs(ctx, &mut self.state);
        chart_panel::show_chart_panel(ctx, &mut self.state);
        stats_panel::show_stats_panel(ctx, &mut self.state);

        if self.state.features.ingest_preview {
            ingest_dialog::show_ingest_dialog(ctx, &mut self.state);
        }
    }
}
