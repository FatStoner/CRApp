use crate::ui::CrapApp;
use eframe::egui;

pub fn render_statistics_window(app: &mut CrapApp, ctx: &egui::Context) {
    if !app.show_statistics_window {
        return;
    }

    let mut open = app.show_statistics_window;
    let window = egui::Window::new("Statistics")
        .open(&mut open)
        .resizable(true)
        .collapsible(false)
        .min_width(400.0)
        .min_height(300.0);

    window.show(ctx, |ui| {
        if let Some(state) = &app.statistics_state {
            ui.heading(&state.source_name);
            ui.add_space(8.0);

            if state.is_calculating {
                ui.horizontal(|ui| {
                    ui.spinner();
                    ui.label("Calcuating statistics...");
                });
            } else if let Some(data) = &state.data {
                ui.label(format!("Character Count: {}", data.character_count));
                ui.add_space(8.0);

                // Main Stats
                ui.group(|ui| {
                    ui.heading("Average Totals");
                    ui.label("Based on current Token options");
                    ui.add_space(4.0);

                    let col_width = (ui.available_width() - 16.0) / 3.0;
                    egui::Grid::new("stats_total_grid")
                        .striped(true)
                        .min_col_width(col_width)
                        .show(ui, |ui| {
                            ui.label("Metric");
                            ui.label("Tokens");
                            ui.label("Characters");
                            ui.end_row();

                            ui.label("Average Per Character");
                            ui.label(format!("{:.1}", data.total_tokens_avg));
                            ui.label(format!("{:.1}", data.total_chars_avg));
                            ui.end_row();
                        });
                });

                ui.add_space(8.0);

                // Breakdown
                ui.group(|ui| {
                    ui.heading("Breakdown by Section");
                    ui.add_space(4.0);

                    let col_width = (ui.available_width() - 16.0) / 3.0;
                    egui::Grid::new("stats_breakdown_grid")
                        .striped(true)
                        .num_columns(3)
                        .min_col_width(col_width)
                        .show(ui, |ui| {
                            ui.label("Section");
                            ui.label("Avg Tokens");
                            ui.label("Avg Characters");
                            ui.end_row();

                            ui.label("Name");
                            ui.label(format!("{:.1}", data.name_tokens_avg));
                            ui.label(format!("{:.1}", data.name_chars_avg));
                            ui.end_row();

                            ui.label("Title / Description");
                            ui.label(format!("{:.1}", data.title_tokens_avg));
                            ui.label(format!("{:.1}", data.title_chars_avg));
                            ui.end_row();

                            ui.label("Personality");
                            ui.label(format!("{:.1}", data.personality_tokens_avg));
                            ui.label(format!("{:.1}", data.personality_chars_avg));
                            ui.end_row();

                            ui.label("Scenario");
                            ui.label(format!("{:.1}", data.scenario_tokens_avg));
                            ui.label(format!("{:.1}", data.scenario_chars_avg));
                            ui.end_row();

                            ui.label("First Message");
                            ui.label(format!("{:.1}", data.first_message_tokens_avg));
                            ui.label(format!("{:.1}", data.first_message_chars_avg));
                            ui.end_row();

                            ui.label("Example Dialogue");
                            ui.label(format!("{:.1}", data.example_dialogue_tokens_avg));
                            ui.label(format!("{:.1}", data.example_dialogue_chars_avg));
                            ui.end_row();
                        });
                });
            } else {
                ui.label("No data available.");
            }
        } else {
            ui.label("Initializing...");
        }
    });

    app.show_statistics_window = open;
}
