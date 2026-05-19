use crate::models::{count_tokens, Character};
use crate::ui::types::{StatisticsData, UiEvent};
use crate::ui::CrapApp;

impl CrapApp {
    pub fn calculate_statistics(&self, characters: Vec<Character>) {
        if characters.is_empty() {
            return;
        }

        let tx = self.tx.clone();

        // Capture settings for "Total" calculation
        let include_name = self.count_name_in_total;
        let include_title = self.count_title_in_total;
        let include_first = self.count_first_message_in_total;
        let include_pers = self.count_personality_in_total;
        let include_scen = self.count_scenario_in_total;
        let include_ex = self.count_example_in_total;

        let ctx = self.ctx.clone();

        crate::task::spawn_supervised(ctx.clone(), async move {
            let count = characters.len();
            if count == 0 {
                return Ok(());
            }

            let mut total_tokens_sum = 0;
            let mut total_chars_sum = 0;

            let mut name_tokens_sum = 0;
            let mut name_chars_sum = 0;

            let mut title_tokens_sum = 0;
            let mut title_chars_sum = 0;

            let mut pers_tokens_sum = 0;
            let mut pers_chars_sum = 0;

            let mut scen_tokens_sum = 0;
            let mut scen_chars_sum = 0;

            let mut first_tokens_sum = 0;
            let mut first_chars_sum = 0;

            let mut ex_tokens_sum = 0;
            let mut ex_chars_sum = 0;

            for char in &characters {
                // Individual Section Calculations
                let t_name = count_tokens(&char.name);
                let c_name = char.name.len();
                name_tokens_sum += t_name;
                name_chars_sum += c_name;

                let t_title = count_tokens(&char.char_title);
                let c_title = char.char_title.len();
                title_tokens_sum += t_title;
                title_chars_sum += c_title;

                let t_pers = count_tokens(&char.personality);
                let c_pers = char.personality.len();
                pers_tokens_sum += t_pers;
                pers_chars_sum += c_pers;

                let t_scen = count_tokens(&char.scenario);
                let c_scen = char.scenario.len();
                scen_tokens_sum += t_scen;
                scen_chars_sum += c_scen;

                let t_first = count_tokens(&char.first_message);
                let c_first = char.first_message.len();
                first_tokens_sum += t_first;
                first_chars_sum += c_first;

                let t_ex = count_tokens(&char.example_dialogue);
                let c_ex = char.example_dialogue.len();
                ex_tokens_sum += t_ex;
                ex_chars_sum += c_ex;

                // Total Calculation based on settings
                let mut char_total_tokens = 0;
                let mut char_total_chars = 0;

                if include_name {
                    char_total_tokens += t_name;
                    char_total_chars += c_name;
                }
                if include_title {
                    char_total_tokens += t_title;
                    char_total_chars += c_title;
                }
                if include_pers {
                    char_total_tokens += t_pers;
                    char_total_chars += c_pers;
                }
                if include_scen {
                    char_total_tokens += t_scen;
                    char_total_chars += c_scen;
                }
                if include_first {
                    char_total_tokens += t_first;
                    char_total_chars += c_first;
                }
                if include_ex {
                    char_total_tokens += t_ex;
                    char_total_chars += c_ex;
                }

                total_tokens_sum += char_total_tokens;
                total_chars_sum += char_total_chars;
            }

            let data = StatisticsData {
                character_count: count,
                total_tokens_avg: total_tokens_sum as f32 / count as f32,
                total_chars_avg: total_chars_sum as f32 / count as f32,

                name_tokens_avg: name_tokens_sum as f32 / count as f32,
                name_chars_avg: name_chars_sum as f32 / count as f32,

                title_tokens_avg: title_tokens_sum as f32 / count as f32,
                title_chars_avg: title_chars_sum as f32 / count as f32,

                personality_tokens_avg: pers_tokens_sum as f32 / count as f32,
                personality_chars_avg: pers_chars_sum as f32 / count as f32,

                scenario_tokens_avg: scen_tokens_sum as f32 / count as f32,
                scenario_chars_avg: scen_chars_sum as f32 / count as f32,

                first_message_tokens_avg: first_tokens_sum as f32 / count as f32,
                first_message_chars_avg: first_chars_sum as f32 / count as f32,

                example_dialogue_tokens_avg: ex_tokens_sum as f32 / count as f32,
                example_dialogue_chars_avg: ex_chars_sum as f32 / count as f32,
            };

            let _ = tx.send(UiEvent::StatisticsCalculated(data)).await;
            ctx.request_repaint();
            Ok(())
        }, self.tx.clone());
    }
}
