use serde::{Deserialize, Serialize};

#[derive(Default, Clone, Debug)]
pub struct ParsedCharacterData {
    pub name: String,
    pub title: String,
    pub personality: String,
    pub scenario: String,
    pub first_message: String,
    pub example_dialogue: String,
    pub external_tags: Vec<String>,
    pub app_tags: Vec<String>,

    pub urls: Vec<crate::models::CharacterUrl>,
    pub avatar_path: Option<String>,
}

#[derive(Default, Clone, Debug, Serialize, Deserialize)]
pub struct ParsedLorebookEntry {
    pub name: String,
    pub keywords: Vec<String>,
    pub content: String,
}

#[derive(Default, Clone, Debug, Serialize, Deserialize)]
pub struct ParsedLorebookData {
    pub title: String,
    pub description: String,
    pub tags: Vec<String>,
    pub entries: Vec<ParsedLorebookEntry>,
}

pub enum ImportFormat {
    Profile,
    Edit,
    CraveEdit,
    GirlfriendGpt,
    JanitorEdit,
    JanitorProfile,

    Unknown,
}

impl ParsedCharacterData {
    pub fn cleanup(&mut self) {
        self.name = self.name.trim().to_string();
        self.title = self.title.trim().to_string();
        self.personality = self.personality.trim().to_string();
        self.scenario = self.scenario.trim().to_string();
        self.first_message = self.first_message.trim().to_string();
        self.example_dialogue = self.example_dialogue.trim().to_string();

        // 1. Remove Advice Lines
        let advice_greeting = "What will they say to start a conversation.";
        let advice_personality = "In a few sentences, describe your chatbot's personality.";
        let advice_scenario = "Describe the current situation and context of the conversation";

        if self.first_message.ends_with(advice_greeting) {
            self.first_message = self
                .first_message
                .trim_end_matches(advice_greeting)
                .trim()
                .to_string();
        }
        if self.personality.ends_with(advice_personality) {
            self.personality = self
                .personality
                .trim_end_matches(advice_personality)
                .trim()
                .to_string();
        }
        if self.scenario.ends_with(advice_scenario) {
            self.scenario = self
                .scenario
                .trim_end_matches(advice_scenario)
                .trim()
                .to_string();
        }

        // 2. Remove Placeholders
        let placeholder_scenario = "Elara Nightshade stands in the center of a dimly lit room, a map of ancient ruins spread out before her. The faint glow from a nearby lantern reflects off the silver streaks in her dark hair as her piercing amber eyes scan the details, her enigmatic presence commanding the air of mystery surrounding the secrets she’s about to uncover.";
        let placeholder_dialogue = "{{User}}: Hey, what are you doing?\n{{Char}}: Greetings {{User}}! I am maintaining SpicyChat’s characters. Pleasure to meet you!\nExample conversations to define your Chatbot. This will impact how it talks.";

        if self.scenario == placeholder_scenario {
            self.scenario = String::new();
        }
        if self.example_dialogue.replace("\r\n", "\n").trim()
            == placeholder_dialogue.replace("\r\n", "\n").trim()
        {
            self.example_dialogue = String::new();
        }
    }
}
