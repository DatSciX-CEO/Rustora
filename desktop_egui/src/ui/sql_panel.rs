use crate::state::AppState;
use crate::ui::theme::BLUE;
use egui::Ui;

pub fn show_sql_panel(ui: &mut Ui, state: &mut AppState) {
    ui.horizontal(|ui| {
        ui.label(egui::RichText::new("SQL").strong().monospace().size(10.0).weak());
        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            ui.label(egui::RichText::new("Ctrl+Enter to run").weak().size(10.0));
        });
    });
    ui.separator();

    ui.horizontal(|ui| {
        let text_edit = egui::TextEdit::multiline(&mut state.sql_text)
            .desired_rows(3)
            .desired_width(ui.available_width() - 80.0)
            .font(egui::TextStyle::Monospace)
            .hint_text("SELECT * FROM table_name WHERE ...");

        let response = ui.add(text_edit);

        let enter_pressed = response.has_focus()
            && ui.input(|i| i.key_pressed(egui::Key::Enter) && i.modifiers.ctrl);

        if ui
            .add_enabled(
                !state.loading && !state.sql_text.trim().is_empty(),
                egui::Button::new(egui::RichText::new("Run").color(BLUE).strong()),
            )
            .clicked()
            || enter_pressed
        {
            state.execute_sql();
        }
    });
}
