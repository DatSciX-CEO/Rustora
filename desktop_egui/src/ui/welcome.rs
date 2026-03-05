use crate::state::AppState;
use crate::ui::theme::ACCENT;
use egui::Ui;

pub fn show_welcome(ui: &mut Ui, state: &mut AppState) {
    ui.vertical_centered(|ui| {
        ui.add_space(80.0);
        ui.heading(egui::RichText::new("Rustora").size(28.0).color(ACCENT).strong());
        ui.add_space(4.0);
        ui.label(
            egui::RichText::new("High-performance data analysis, 100% on your machine")
                .size(14.0)
                .weak(),
        );
        ui.add_space(24.0);

        let button_width = 320.0;

        ui.allocate_ui_with_layout(
            egui::vec2(button_width, 0.0),
            egui::Layout::top_down(egui::Align::Center),
            |ui| {
                if ui
                    .add_sized(
                        [button_width, 48.0],
                        egui::Button::new(
                            egui::RichText::new("  New Project  --  Create a .duckdb project")
                                .size(13.0),
                        ),
                    )
                    .clicked()
                {
                    if let Some(path) = rfd::FileDialog::new()
                        .set_title("Create New Rustora Project")
                        .add_filter("Rustora Project", &["duckdb"])
                        .save_file()
                    {
                        state.new_project(&path.to_string_lossy());
                    }
                }

                ui.add_space(4.0);

                if ui
                    .add_sized(
                        [button_width, 48.0],
                        egui::Button::new(
                            egui::RichText::new("  Open Project  --  Resume an existing project")
                                .size(13.0),
                        ),
                    )
                    .clicked()
                {
                    if let Some(path) = rfd::FileDialog::new()
                        .set_title("Open Rustora Project")
                        .add_filter("Rustora Project", &["duckdb"])
                        .pick_file()
                    {
                        state.open_project(&path.to_string_lossy());
                    }
                }

                ui.add_space(4.0);

                if ui
                    .add_sized(
                        [button_width, 48.0],
                        egui::Button::new(
                            egui::RichText::new("  Open File  --  Quick-analyze CSV, Parquet, or Arrow")
                                .size(13.0),
                        ),
                    )
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
                        if state.project.is_some() {
                            state.import_file(&path_str);
                        } else {
                            state.open_file(&path_str);
                        }
                    }
                }
            },
        );

        ui.add_space(16.0);
        ui.label(egui::RichText::new("Use the toolbar above or click a card to get started").weak().size(11.0));
    });
}
