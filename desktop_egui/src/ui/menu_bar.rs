use crate::state::AppState;
use crate::ui::theme::{ACCENT, SUCCESS};
use egui::Ui;

pub fn show_menu_bar(ui: &mut Ui, state: &mut AppState) {
    ui.horizontal(|ui| {
        ui.label(egui::RichText::new("Rustora").strong().color(ACCENT).size(14.0));
        ui.separator();

        if let Some(ref proj) = state.project {
            let proj_name = std::path::Path::new(&proj.path)
                .file_stem()
                .map(|s| s.to_string_lossy().to_string())
                .unwrap_or_else(|| proj.path.clone());
            ui.label(egui::RichText::new(format!("Project: {}", proj_name)).color(SUCCESS).size(11.0));
            ui.separator();
        }

        // File group
        if ui.button("New Project").clicked() {
            if let Some(path) = rfd::FileDialog::new()
                .set_title("Create New Rustora Project")
                .add_filter("Rustora Project", &["duckdb"])
                .save_file()
            {
                state.new_project(&path.to_string_lossy());
            }
        }

        if ui.button("Open Project").clicked() {
            if let Some(path) = rfd::FileDialog::new()
                .set_title("Open Rustora Project")
                .add_filter("Rustora Project", &["duckdb"])
                .pick_file()
            {
                state.open_project(&path.to_string_lossy());
            }
        }

        let import_label = if state.project.is_some() {
            "Import File"
        } else {
            "Open File"
        };
        if ui
            .button(egui::RichText::new(import_label).color(ACCENT))
            .clicked()
        {
            if let Some(path) = rfd::FileDialog::new()
                .set_title("Open Data File")
                .add_filter(
                    "Data Files",
                    &["csv", "tsv", "parquet", "pq", "ipc", "arrow", "feather"],
                )
                .pick_file()
            {
                let path_str = path.to_string_lossy().to_string();
                if state.features.ingest_preview && state.project.is_some() {
                    state.start_ingest(&path_str);
                } else if state.project.is_some() {
                    state.import_file(&path_str);
                } else {
                    state.open_file(&path_str);
                }
            }
        }

        ui.separator();

        // Analysis group
        let sql_label = if state.sql_visible { "Hide SQL" } else { "SQL" };
        if ui.button(sql_label).clicked() {
            state.sql_visible = !state.sql_visible;
        }

        if ui
            .add_enabled(state.name.is_some(), egui::Button::new("Filter"))
            .clicked()
        {
            state.filter_dialog = true;
        }

        if ui
            .add_enabled(state.name.is_some(), egui::Button::new("Group By"))
            .clicked()
        {
            state.group_dialog = true;
        }

        if ui
            .add_enabled(state.name.is_some(), egui::Button::new("Add Column"))
            .clicked()
        {
            state.calc_dialog = true;
        }

        if state.features.column_ops {
            if ui
                .add_enabled(state.name.is_some(), egui::Button::new("Rename Col"))
                .clicked()
            {
                state.rename_dialog = true;
            }
        }

        if ui
            .add_enabled(state.name.is_some(), egui::Button::new("Stats"))
            .clicked()
        {
            state.load_summary_stats();
        }

        if state.features.pivot_unpivot {
            if ui
                .add_enabled(state.name.is_some(), egui::Button::new("Pivot"))
                .clicked()
            {
                state.pivot_dialog = true;
            }
            if ui
                .add_enabled(state.name.is_some(), egui::Button::new("Unpivot"))
                .clicked()
            {
                state.unpivot_dialog = true;
            }
        }

        if state.features.merge_append {
            if ui
                .add_enabled(state.name.is_some(), egui::Button::new("Merge"))
                .clicked()
            {
                state.merge_dialog = true;
            }
            if ui
                .add_enabled(!state.tables.is_empty(), egui::Button::new("Append"))
                .clicked()
            {
                state.append_dialog = true;
            }
        }

        if state.features.steps_panel {
            let steps_label = if state.steps_visible {
                "Hide Steps"
            } else {
                "Steps"
            };
            if ui.button(steps_label).clicked() {
                state.steps_visible = !state.steps_visible;
            }
        }

        let chart_label = if state.chart_visible {
            "Hide Chart"
        } else {
            "Chart"
        };
        if ui
            .add_enabled(state.name.is_some(), egui::Button::new(chart_label))
            .clicked()
        {
            state.chart_visible = !state.chart_visible;
            if state.chart_visible {
                if let Some(ref col) = state.columns.first() {
                    if state.chart_group_col.is_empty() {
                        state.chart_group_col = col.name.clone();
                    }
                }
                if state.columns.len() > 1 && state.chart_value_col.is_empty() {
                    state.chart_value_col = state.columns[1].name.clone();
                }
            }
        }

        ui.separator();

        // Export group
        if ui
            .add_enabled(state.name.is_some(), egui::Button::new("Export CSV"))
            .clicked()
        {
            if let Some(path) = rfd::FileDialog::new()
                .set_title("Export as CSV")
                .add_filter("CSV", &["csv"])
                .save_file()
            {
                state.export_dataset(&path.to_string_lossy(), "csv");
            }
        }

        if ui
            .add_enabled(state.name.is_some(), egui::Button::new("Export Parquet"))
            .clicked()
        {
            if let Some(path) = rfd::FileDialog::new()
                .set_title("Export as Parquet")
                .add_filter("Parquet", &["parquet"])
                .save_file()
            {
                state.export_dataset(&path.to_string_lossy(), "parquet");
            }
        }

        ui.separator();

        // Theme toggle
        let dark_mode = ui.visuals().dark_mode;
        let theme_label = if dark_mode { "Light" } else { "Dark" };
        if ui.button(theme_label).clicked() {
            let visuals = if dark_mode {
                egui::Visuals::light()
            } else {
                egui::Visuals::dark()
            };
            ui.ctx().set_visuals(visuals);
            crate::ui::theme::apply_rustora_theme(ui.ctx());
        }

        // Loading indicator
        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            if state.loading {
                ui.spinner();
                ui.label(egui::RichText::new("Loading...").color(crate::ui::theme::WARNING).size(11.0));
            }
            if let Some(ref name) = state.name {
                ui.label(
                    egui::RichText::new(name.as_str())
                        .size(11.0)
                        .monospace()
                        .weak(),
                );
            }
        });
    });
}
