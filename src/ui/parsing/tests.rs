use super::spicychat::parse_spicychat_lorebook;
use super::types::*;
use super::utils::*;

#[test]
fn test_spicy_cleanup_advice_lines() {
    let mut data = ParsedCharacterData {
        first_message: "Hello user!\nWhat will they say to start a conversation.".to_string(),
        personality: "Kind bot.\nIn a few sentences, describe your chatbot's personality."
            .to_string(),
        scenario: "In a park.\nDescribe the current situation and context of the conversation"
            .to_string(),
        ..Default::default()
    };
    data.cleanup();
    assert_eq!(data.first_message, "Hello user!");
    assert_eq!(data.personality, "Kind bot.");
    assert_eq!(data.scenario, "In a park.");
}

#[test]
fn test_multiline_preservation() {
    let raw_text = "Edit Chatbot\nName\nTestBot\nPersonality\nPara 1.\n\nPara 2.\ntokens: 100";
    let data = parse_clipboard(raw_text);
    assert_eq!(data.name, "TestBot");
    assert!(data.personality.contains("Para 1.\n\nPara 2."));
}

#[test]
fn test_edit_view_with_empty_lines() {
    let raw_text = "Edit Chatbot\nName\n\nTestBot\nTitle\n\nThe Title\nPersonality\nDesc";
    let data = parse_clipboard(raw_text);
    assert_eq!(data.name, "TestBot");
    assert_eq!(data.title, "The Title");
}

#[test]
fn test_profile_view_loose_structure_with_empty_name_gap() {
    let raw_text = "
        Back
        
        avatar image
        
        MyName
        Suggest Tag
        ";
    let data = parse_clipboard(raw_text);
    assert_eq!(data.name, "MyName");
}

#[test]
fn test_parse_ggpt_view() {
    let raw_text = r#"GirlfriendGPT
Edit Character
Character Name
Kaida

Description (272 tokens)
Sunlight filters through the curtains.
Write a brief overview of your character.

Personality (938 tokens)
Kaida Akiko
Age: 22
Describe your character's traits, behavior, and demeanor.

Scenario (674 tokens)
RULES: always use DESCRIPTIONS
Legacy

First Message (475 tokens)
Warm sunlight filtered through.
Legacy

Example Conversation(2 tokens)
.
⚠️ Can cause unpredictable behavior, use with care.

Character Tags
Add tag
Female
Original Character (OC)
Assign tags that describes your character.
"#;
    let data = parse_clipboard(raw_text);
    assert_eq!(data.name, "Kaida");
    assert!(data.title.contains("Sunlight filters through"));
    assert!(!data.title.contains("Write a brief overview"));
    assert!(data.personality.contains("Kaida Akiko"));
    assert!(data.scenario.contains("RULES: always use DESCRIPTIONS"));
    assert!(data
        .first_message
        .contains("Warm sunlight filtered through."));
    assert!(data.example_dialogue.contains("."));
    assert!(!data.example_dialogue.contains("⚠️"));
    assert!(data.external_tags.contains(&"Female".to_string()));
    assert!(data
        .external_tags
        .contains(&"Original Character (OC)".to_string()));
}

#[test]
fn test_profile_with_lorebook_spacing() {
    let raw_text = "
        Back
        
        avatar image
        BotName
        
        Share
        
        100 tokens
        
        MyLorebook
        
        MyTitle
        
        
        MyTag
        Suggest Tag
        ";
    let data = parse_clipboard(raw_text);
    assert_eq!(data.name, "BotName");
    assert_eq!(data.title, "MyTitle");
    assert_eq!(data.external_tags, vec!["MyTag"]);
}

#[test]
fn test_profile_without_lorebook_spacing() {
    let raw_text = "
        Back
        
        avatar image
        BotName
        
        Share
        
        100 tokens
        
        MyTitle
        
        
        MyTag
        Suggest Tag
        ";
    let data = parse_clipboard(raw_text);
    assert_eq!(data.name, "BotName");
    assert_eq!(data.title, "MyTitle");
    assert_eq!(data.external_tags, vec!["MyTag"]);
}

#[test]
fn test_edit_view_tags_with_empty_lines() {
    let raw_text = "
        Edit Chatbot
        Name
        Bot
        Tags
        
        Tag1
        
        Tag2
        1/12
        ";
    let data = parse_clipboard(raw_text);
    assert_eq!(data.name, "Bot");
    assert_eq!(data.external_tags, vec!["Tag1", "Tag2"]);
}

#[test]
fn test_parse_crave_edit_view() {
    let raw_text = "Edit Characters | CraveU AI
Character Name*
Anya
Introduction*
The rain was coming down in sheets...
Personality*
ANYA{name: Anya. idea: A young woman...}
688 tokens
Tags*
Female
Adventure
OC
Initial Message (Greeting)*
The rain was coming down in sheets, turning the city streets into a maze of mirrored black. {{user}} hurried along...
Scenario
RULES: Only user can control {{user}} actions...
604 tokens
Save";
    let data = parse_clipboard(raw_text);
    assert_eq!(data.name, "Anya");
    assert!(data.title.contains("The rain was coming down in sheets..."));
    assert!(data
        .personality
        .contains("ANYA{name: Anya. idea: A young woman...}"));
    assert_eq!(data.external_tags, vec!["Female", "Adventure", "OC"]);
    assert!(data
        .first_message
        .contains("The rain was coming down in sheets"));
    assert!(data.scenario.contains("RULES: Only user can control"));
}

#[test]
fn test_parse_janitor_edit() {
    let raw_text = r#"janitor
beta

Search characters...
Create a Character

Edit Character (View Character)
Image

No file chosen
Preview
Character Name*
Alexandra Jones
Character Bio
Paragraph
The moment you step into your sunlit apartment...
This will be displayed in your character card

Character Settings

Character Tags
Winter Holidays 2025 Event
Female

Personality*
name: Alexandra Jones
idea: Submissive roommate
Scenario
RULES: always use DESCRIPTIONS
Initial messages (first messages) *(1/10)
First message from your character.
Sunlight streamed through the living room window...

Example dialogs
{{char}}: Hey
{{user}}: Hello
"#;
    let data = parse_clipboard(raw_text);
    assert_eq!(data.name, "Alexandra Jones");
    assert!(data.title.contains("The moment you step into"));
    assert!(!data.title.contains("This will be displayed"));
    assert!(data.personality.contains("name: Alexandra Jones"));
    assert!(data.scenario.contains("RULES: always use DESCRIPTIONS"));
    assert!(data.first_message.contains("Sunlight streamed"));
    assert!(data.example_dialogue.contains("{{char}}: Hey"));
    assert!(data.external_tags.is_empty());
}

#[test]
fn test_parse_janitor_profile() {
    let raw_text = r#"janitor
beta

Analytics
beta

Saira
Saira
0

1

by:
@JustJam

[Master-servant, Fantasy, Middle-east]

The workshop's familiar scent...
Created Feb 16, 2025

Personality (654 tokens)
Saira Idea: The Guildmaster's Daughter...
Scenario (576 tokens)
RULES: always...
First Message (405 tokens)
The air in the silversmith workshop...
Example Dialogs (0 tokens)
0
comments
"#;
    let data = parse_clipboard(raw_text);
    assert_eq!(data.name, "Saira");
    assert!(data.title.contains("The workshop's familiar scent"));
    assert!(data.personality.contains("Saira Idea:"));
    assert!(data.scenario.contains("RULES: always"));
    assert!(data.first_message.contains("The air in the silversmith"));
    assert!(data.external_tags.contains(&"Fantasy".to_string()));
}

#[test]
fn test_parse_spicychat_lorebook() {
    let html = r#"
            <div class="w-full mx-auto max-w-[750px] bg-gray-1 dark:bg-gray-3 rounded-lg p-3 md:p-4 border border-solid border-gray-5">
                <div class="flex justify-between max-mob:flex-col ">
                    <div class="flex items-center gap-sm">
                        <p class="font-sans text-decoration-skip-ink-none text-underline-position-from-font text-heading-5 font-bold text-left flex gap-sm items-center">Edit Lorebook</p>
                    </div>
                </div>
                <p class="font-sans text-decoration-skip-ink-none text-underline-position-from-font text-label-lg font-medium text-left text-gray-11 mt-sm">Test Lorebook #01</p>
                <div class="py-xl flex flex-col gap-xl pb-0 pt-lg">
                    <div class="flex flex-col">
                        <button type="button" class="w-full flex items-center justify-between rounded-lg cursor-pointer transition-colors duration-200 bg-transparent border border-solid border-transparent gap-2 py-md px-[13px] min-h-auto hover:bg-gray-4">
                            <div class="flex items-center gap-md flex-1">
                                <div class="flex flex-col min-w-0 flex-1 gap-0.5">
                                    <p class="font-sans text-decoration-skip-ink-none text-underline-position-from-font text-label-md font-regular text-left text-gray-12 line-clamp-2">Example entry 1</p>
                                    <div class="flex items-center gap-1">
                                        <p class="font-sans text-decoration-skip-ink-none text-underline-position-from-font text-label-sm font-regular text-left text-gray-11 line-clamp-1">keyword_example1</p>
                                    </div>
                                    <p class="font-sans text-decoration-skip-ink-none text-underline-position-from-font text-label-sm font-regular text-left" style="display: -webkit-box; -webkit-box-orient: vertical; overflow: hidden; -webkit-line-clamp: 2; text-overflow: ellipsis;">Lorem ipsum dolor sit amet</p>
                                </div>
                            </div>
                        </button>
                        <button type="button" class="w-full flex items-center justify-between rounded-lg cursor-pointer transition-colors duration-200 bg-transparent border border-solid border-transparent gap-2 py-md px-[13px] min-h-auto hover:bg-gray-4">
                            <div class="flex items-center gap-md flex-1">
                                <div class="flex flex-col min-w-0 flex-1 gap-0.5">
                                    <p class="font-sans text-decoration-skip-ink-none text-underline-position-from-font text-label-md font-regular text-left text-gray-12 line-clamp-2">Example entry 2</p>
                                    <div class="flex items-center gap-1">
                                        <p class="font-sans text-decoration-skip-ink-none text-underline-position-from-font text-label-sm font-regular text-left text-gray-11 line-clamp-1">2example_keyword</p>
                                    </div>
                                    <p class="font-sans text-decoration-skip-ink-none text-underline-position-from-font text-label-sm font-regular text-left" style="display: -webkit-box; -webkit-box-orient: vertical; overflow: hidden; -webkit-line-clamp: 2; text-overflow: ellipsis;">Second entry content</p>
                                </div>
                            </div>
                        </button>
                    </div>
                </div>
            </div>
            "#;

    let parsed = parse_spicychat_lorebook(html);

    assert_eq!(parsed.title, "Test Lorebook #01");
    assert_eq!(parsed.entries.len(), 2);

    let entry1 = &parsed.entries[0];
    assert_eq!(entry1.name, "Example entry 1");
    assert_eq!(entry1.keywords, vec!["keyword_example1"]);
    assert_eq!(entry1.content, "Lorem ipsum dolor sit amet");

    let entry2 = &parsed.entries[1];
    assert_eq!(entry2.name, "Example entry 2");
    assert_eq!(entry2.keywords, vec!["2example_keyword"]);
    assert_eq!(entry2.content, "Second entry content");
}

#[test]
fn test_parse_spicychat_lorebook_profile_view() {
    let html = r#"
            <div class="some-container">
                <p class="text-mobile-heading-3 font-bold">Public Lorebook Title</p>
                <div class="author-section">
                    <p class="text-label-lg">@AuthorName</p>
                </div>
                <div class="desc-section">
                    <p class="text-label-lg">This is a description of the lorebook.</p>
                </div>
                
                <!-- Navbar element that mimics entry button but lacks line-clamp-2 -->
                <button class="hover:bg-gray-4 flex items-center">
                    <div class="flex items-center gap-2">
                         <p class="text-label-lg">Home</p>
                    </div>
                </button>

                <div class="w-full">
                    <p class="text-heading-5">Entries</p>
                    <div class="entries-list">
                        <button class="hover:bg-gray-4 flex items-center">
                            <div class="entry-content">
                                <p class="text-gray-12 text-label-md line-clamp-2">Profile Entry 1</p>
                                <p class="text-gray-11 line-clamp-1">profile_kw1, profile_kw2</p>
                                <p class="style-clamp" style="-webkit-line-clamp: 2">Profile Content 1</p>
                            </div>
                        </button>
                    </div>
                </div>
            </div>
        "#;

    let parsed = parse_spicychat_lorebook(html);

    assert_eq!(parsed.title, "Public Lorebook Title");
    assert_eq!(parsed.description, "This is a description of the lorebook.");

    // Should only find 1 entry, ignoring the Navbar element
    assert_eq!(parsed.entries.len(), 1);
    assert_eq!(parsed.entries[0].name, "Profile Entry 1");
    assert_eq!(
        parsed.entries[0].keywords,
        vec!["profile_kw1", "profile_kw2"]
    );
    assert_eq!(parsed.entries[0].content, "Profile Content 1");
}
