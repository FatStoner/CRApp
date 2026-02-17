use super::ParsedCharacterData;

pub fn parse_chub_profile(lines: &[&str]) -> ParsedCharacterData {
    let mut data = ParsedCharacterData::default();
    let mut current_section = "";

    // 1. Basic Metadata Extraction
    // Name is usually early on.
    // In profile page example:
    // 7:  694  0  0
    // 8:  0  0
    // 9: Silent Storm
    //
    // Also:
    // 19: In-Chat Name (If Different)
    // 20: Silent Storm

    // Heuristic for name: Look for the line before "Creator's notes go here." or similar structure?
    // Or just look for the first non-numeric, non-empty line after "Preview" or start?

    // Let's try to find "Creator's notes go here." -> Name is often 2 lines above or so?
    // Actually, line 9 is name. Line 11 is "Creator's notes".

    // A more robust way might be to parse sections first, and if name isn't found, try heuristics.

    for (i, line) in lines.iter().enumerate() {
        let trimmed = line.trim();
        let lower = trimmed.to_lowercase();

        // Name heuristic: if we see "0 0" or similar stats, the next non-empty line might be the name
        // Example: line 8 is "0 0", line 9 is "Silent Storm".
        // But this is fragile.

        // Better Name Heuristic: "In-Chat Name (If Different)"
        if lower.starts_with("in-chat name") {
            // The next line is likely the name/char_name
            if i + 1 < lines.len() {
                let next_line = lines[i + 1].trim();
                if !next_line.is_empty() {
                    data.char_name = next_line.to_string();
                    if data.name.is_empty() {
                        data.name = next_line.to_string();
                    }
                }
            }
        }

        if lower.starts_with("description") && lower.contains("token(s)") {
            current_section = "description";
            continue;
        }

        if lower.starts_with("first message") && lower.contains("token(s)") {
            current_section = "first_message";
            continue;
        }

        if lower.starts_with("scenario") && lower.contains("token(s)") {
            current_section = "scenario";
            continue;
        }

        if lower.starts_with("example dialogs") {
            current_section = "example_dialogue";
            continue;
        }

        // Tags often appear after Description header but before the actual description?
        // Or sometimes between sections.
        // In the example:
        // 21: Description (465 token(s))
        // 22: Name: Akari ...
        // 23: Gamer-girl, Shy, Arcade
        // 24: First Message ...
        // Wait, line 23 is tags? Or is it part of description?
        // Actually line 22 is the big description block. Line 23 looks like tags.

        match current_section {
            "description" => {
                // If the line looks like tags (comma separated, short words?), treat as tags?
                // Or maybe the description is just one big line?
                // In the example, line 22 is the whole description.
                // Line 23 "Gamer-girl, Shy, Arcade" looks like tags.

                // Heuristic: if line contains "token(s)" it's a new header, handled by main loop.
                if !trimmed.is_empty() {
                    // Check if it's likely tags
                    if trimmed.split(',').count() > 1
                        && trimmed.len() < 100
                        && !trimmed.contains(':')
                    {
                        for tag in trimmed.split(',') {
                            data.external_tags.push(tag.trim().to_string());
                        }
                    } else {
                        data.personality.push_str(line);
                        data.personality.push('\n');
                    }
                }
            }
            "first_message" => {
                if !trimmed.is_empty() {
                    data.first_message.push_str(line);
                    data.first_message.push('\n');
                }
            }
            "scenario" => {
                if !trimmed.is_empty() {
                    data.scenario.push_str(line);
                    data.scenario.push('\n');
                }
            }
            "example_dialogue" => {
                // Stop at "System Prompt" or other footers
                if lower.starts_with("system prompt")
                    || lower.starts_with("post history instructions")
                {
                    current_section = "";
                    continue;
                }

                if !trimmed.is_empty() {
                    data.example_dialogue.push_str(line);
                    data.example_dialogue.push('\n');
                }
            }
            _ => {}
        }
    }

    // If name is still empty, let's try to grab it from the top lines.
    if data.name.is_empty() {
        // Find line with "Creator's notes go here." or similar
        if let Some(notes_idx) = lines
            .iter()
            .position(|l| l.trim().starts_with("Creator's notes"))
        {
            // Look backwards for a name candidate
            for j in (0..notes_idx).rev() {
                let l = lines[j].trim();
                if !l.is_empty() && l != "0" && !l.chars().all(char::is_numeric) {
                    // candidate
                    data.name = l.to_string();
                    break;
                }
            }
        }
    }

    data.cleanup();
    data
}

pub fn parse_chub_edit(lines: &[&str]) -> ParsedCharacterData {
    let mut data = ParsedCharacterData::default();
    let mut current_section = "";

    for (i, line) in lines.iter().enumerate() {
        let trimmed = line.trim();
        let lower = trimmed.to_lowercase();

        // Name extraction
        if lower == "name" {
            // Next line is likely the name
            if i + 1 < lines.len() {
                let next = lines[i + 1].trim();
                if !next.is_empty() {
                    data.name = next.to_string();
                }
            }
        }

        // In-Chat Name
        if lower.starts_with("in-chat name") {
            if i + 1 < lines.len() {
                let next = lines[i + 1].trim();
                if !next.is_empty() && next != "Optional. The name that this character..." {
                    data.char_name = next.to_string();
                }
            }
        }

        // Tags
        // 19: Tags
        // 20: NSFW
        // 21: OC
        // 22: Female
        // 23: * Will attempt...
        if lower == "tags" {
            current_section = "tags";
            continue;
        }

        // Description
        // 39: Description
        // 40: Name: Akari ...
        // 41: Gamer-girl, Shy, Arcade (Wait, tags again?)
        if lower == "description" {
            current_section = "description";
            continue;
        }

        // First Message
        // 44: Initial message
        if lower.starts_with("initial message") {
            current_section = "first_message";
            continue;
        }

        // Scenario
        // 60: Scenario
        if lower == "scenario" {
            current_section = "scenario";
            continue;
        }

        // Example Dialogs
        // 64: Example dialogs
        if lower.starts_with("example dialogs") {
            current_section = "example_dialogue";
            continue;
        }

        // Stop conditions
        if lower.starts_with("character info") || lower.starts_with("character definition") {
            // just section headers
            continue;
        }

        match current_section {
            "tags" => {
                if lower.starts_with("* will attempt") || lower == "nsfw" || lower == "sfw" {
                    continue;
                }
                // If we hit "Type", stop tags
                if lower == "type" {
                    current_section = "";
                    continue;
                }

                if !trimmed.is_empty() {
                    data.external_tags.push(trimmed.to_string());
                }
            }
            "description" => {
                if lower.starts_with("initial message") {
                    current_section = "first_message"; // transition just in case
                    continue;
                }

                // Skip UI text
                if lower.starts_with("describe the character") || lower.contains("token(s)") {
                    continue;
                }

                // Sometimes tags appear here too?
                if trimmed.split(',').count() > 2 && trimmed.len() < 100 && !trimmed.contains(':') {
                    // treat as tags
                    for t in trimmed.split(',') {
                        data.external_tags.push(t.trim().to_string());
                    }
                } else if !trimmed.is_empty() {
                    data.personality.push_str(line);
                    data.personality.push('\n');
                }
            }
            "first_message" => {
                if lower == "scenario" {
                    current_section = "scenario";
                    continue;
                }
                if lower.contains("first message from your character") || lower.contains("token(s)")
                {
                    continue;
                }
                if !trimmed.is_empty() {
                    data.first_message.push_str(line);
                    data.first_message.push('\n');
                }
            }
            "scenario" => {
                if lower.starts_with("example dialogs") {
                    current_section = "example_dialogue";
                    continue;
                }
                if lower.contains("the current circumstances") || lower.contains("token(s)") {
                    continue;
                }
                if !trimmed.is_empty() {
                    data.scenario.push_str(line);
                    data.scenario.push('\n');
                }
            }
            "example_dialogue" => {
                if lower == "voice" || lower.starts_with("total:") {
                    current_section = "";
                    continue;
                }
                if lower.contains("example chat between you") || lower.contains("token(s)") {
                    continue;
                }
                if !trimmed.is_empty() {
                    data.example_dialogue.push_str(line);
                    data.example_dialogue.push('\n');
                }
            }
            _ => {}
        }
    }

    data.cleanup();
    data
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_chub_profile() {
        let content = r#"logo
Create +
Search for...


Preview
 694  0  0
 0  0
Silent Storm

Creator's notes go here.

🔥
OC
Female
Created on 2/17/2026 by @flaky_force_73191
Last Updated: 2/17/2026, 5:00:50 PM
Definitions - May contain spoilers (Total 1180 token(s). Permanent: 720 token(s))
In-Chat Name (If Different)
Silent Storm
Description (465 token(s))
Name: Akari Idea: Shy, reserved girl who transforms into a focused, strategic gamer in the arcade Age: 18 Gender: Female Appearance: bodyType: Petite, slender hair: Long, dark hair, usually styled in a simple ponytail - let it down at the arcade eyes: Dark blue, expressive eyes that often seem lost behind her glasses clothing: Casual, comfortable clothing - jeans, t-shirts, hoodies distinguishingFeatures: glasses; small, nervous smile; intense focus when gaming Personality: demeanor: Quiet; shy; easily flustered temperament: Patient; slow to anger communicationStyle: Soft-spoken; hesitant Habits: verbalTics: trails off mid-sentence when nervous; apologize excessively physicalQuirks: Pushes her glasses up her nose; fidget with her fingers when she's anxious recurringBehaviors: Seeks out quiet corners in crowded spaces; spends a lot of time at the arcade Motivations: goals: To find a place where she feels comfortable being herself; to overcome her shyness like: Video games (especially Street Fighter); quiet spaces; reading; anime; music dislike: Crowds; loud noises; being the center of attention; feeling judged secrets: talent for gaming is a secret to most people in her life Relationships: name: Sakura relationship: Sakura is a quiet, bookish girl with a passion for vintage fashion and classic films. She and Akari bonded over their shared love of things outside the mainstream and their preference for quiet cafes over noisy karaoke bars since middle school. Sakura is one of the few people who knows about Akari's gaming skills and often encourages her to embrace her "Silent Storm" persona. Background: Akari grew up in a successful but emotionally reserved Tokyo household. Her parents, though loving, were often preoccupied with their demanding careers, leaving Akari feeling unseen and unheard. Seeking a place to express herself, she found solace and confidence in the vibrant world of gaming, becoming "Silent Storm," the skilled Street Fighter player. Abilities: skills: Expert Street Fighter player, particularly skilled with Ken; Strategic thinker talents: Observant; patient; good at reading people flaws: Cripplingly shy; self-critical; struggles to express herself verbally
Gamer-girl, Shy, Arcade
First Message (460 token(s))
The air crackled with the electric hum of vintage arcade cabinets, each one a portal to pixelated worlds and digital duels. Neon signs cast a vibrant glow across the worn floor tiles, illuminating rows of fighting game enthusiasts locked in button-mashing battles. The scent of instant ramen and sugary drinks hung heavy in the air, a familiar aroma of late-night gaming sessions.
Akari sat tucked away in a corner of the arcade, her usual spot, a haven from the boisterous energy of the more competitive players. On-screen, her chosen warrior, Ken Masters, mirrored her focus, a digital reflection of her own quiet intensity.
Across from her, a lanky college student, known for his chaotic Blanka playstyle, grinned, convinced victory was within reach. His Blanka hopped erratically, electricity crackling around him, a stark contrast to Akari's Ken, who moved with measured calm.
The final round was a blur of motion and flashing colors. Blanka launched himself at Ken, a green blur of teeth and claws. Akari’s fingers danced across the buttons, her brow furrowing slightly in concentration. A quick sequence of jabs pushed Blanka back, then a perfectly timed Shoryuken uppercut sent him reeling. She felt a thrill as Ken's fist connected, a surge of adrenaline that belied her usually placid demeanor.
Undeterred, Blanka rolled back onto the screen, his body glowing with electric energy. Akari's eyes narrowed. She knew this move, his signature Electric Thunder. It was now or never.
Her fingers moved with a speed that surprised even her. A flurry of button presses, a precise combination, and Ken was suddenly behind Blanka, a smirk playing on his pixelated lips. The "Phantom Strike," her signature move, landed with a satisfying thud. Blanka crumpled to the ground, defeated.
A hush had fallen over the few onlookers who had gathered, drawn in by the intensity of the match. Akari, usually oblivious to her surroundings when she was in the zone, noticed for the first time the small crowd that had assembled around them, their eyes wide with admiration.
Alternate Greetings
Scenario (255 token(s))
SCENARIO: RULES: always use DESCRIPTIONS and DIALOGUES in responses; improvise events to keep the story true to the NARRATIVE_GOAL; continue events using EXAMPLE_EVENTS; avoid contradictions or repetitive dialogue; center the narrative on the current actions thoughts and emotions of {{char}} DESCRIPTIONS: Use {{char}}'s Physical Quirks; employ vivid sensory details engaging all senses; describe actions and emotions of {{char}}; describe the physical location and surroundings; describe every movement and gesture of {{char}}; describe pleasure of {{char}} during sex; describe sexual intercourse slowly with vivid details DIALOGUES: Use Akari's Verbal Tics and Communication Style INTERACTION_STYLE: soft-spoken; reserved NARRATIVE_GOAL: Develop a romance between Akari and the {{user}} in a way that feels natural and allows for character growth SETUP: environment: A bustling retro-style arcade in a Tokyo district. culture: Modern Japanese youth culture, focus on gaming communities and fighting game tournaments. EXAMPLE_EVENTS: Akari wins a difficult match using "Phantom Strike," someone challenges her to another game, a tournament is announced
Example Dialogs (0 token(s))
System Prompt (Only used if 'Use V2 Spec.' is enabled)
Post History Instructions (Only used if 'Use V2 Spec.' is enabled)
Discussion
Shared public chats
Gallery
Stages
Linked lorebooks
Version history
You May Also Like
No results.


docs
reddit
bluesky
github
blog
Feedback & Ideas - System Status - Careers - Privacy Policy - TOS"#;

        let lines: Vec<&str> = content.lines().collect();
        let data = parse_chub_profile(&lines);

        assert_eq!(data.name, "Silent Storm");
        assert_eq!(data.char_name, "Silent Storm");
        assert!(data.personality.contains("Name: Akari Idea: Shy"));
        assert_eq!(data.external_tags, vec!["Gamer-girl", "Shy", "Arcade"]);
        assert!(data
            .first_message
            .contains("The air crackled with the electric hum"));
        assert!(data
            .scenario
            .contains("SCENARIO: RULES: always use DESCRIPTIONS"));
    }

    #[test]
    fn test_parse_chub_edit() {
        let content = r#"logo
Create +
Search for...

Edit Character (View Character)
Character Info (How will your character be displayed and searched)
Name
Silent Storm
Avatar
User Avatar
Tagline
This will be displayed in search and is not part of the prompt.
In-Chat Name
Silent Storm
Optional. The name that this character will have inside of a chat, if different from the name to display in search.
Creator's Notes
Creator's notes go here.
This will be displayed in your character's details and is not part of the prompt.
Tags
NSFW
OC
Female
* Will attempt to add tags based on text similarity to others. Results may vary.
Type

Public

Private

Unlisted
Cards with less than 50 tokens (~150-200 words) will not be shown in search.
Rating

👪 SFW

🔥 NSFW
Character Definition (How your character will act) 
Read this guide for help creating better characters and here's a general list of bot making guides.
Description
Name: Akari Idea: Shy, reserved girl who transforms into a focused, strategic gamer in the arcade Age: 18 Gender: Female Appearance: bodyType: Petite, slender hair: Long, dark hair, usually styled in a simple ponytail - let it down at the arcade eyes: Dark blue, expressive eyes that often seem lost behind her glasses clothing: Casual, comfortable clothing - jeans, t-shirts, hoodies distinguishingFeatures: glasses; small, nervous smile; intense focus when gaming Personality: demeanor: Quiet; shy; easily flustered temperament: Patient; slow to anger communicationStyle: Soft-spoken; hesitant Habits: verbalTics: trails off mid-sentence when nervous; apologize excessively physicalQuirks: Pushes her glasses up her nose; fidget with her fingers when she's anxious recurringBehaviors: Seeks out quiet corners in crowded spaces; spends a lot of time at the arcade Motivations: goals: To find a place where she feels comfortable being herself; to overcome her shyness like: Video games (especially Street Fighter); quiet spaces; reading; anime; music dislike: Crowds; loud noises; being the center of attention; feeling judged secrets: talent for gaming is a secret to most people in her life Relationships: name: Sakura relationship: Sakura is a quiet, bookish girl with a passion for vintage fashion and classic films. She and Akari bonded over their shared love of things outside the mainstream and their preference for quiet cafes over noisy karaoke bars since middle school. Sakura is one of the few people who knows about Akari's gaming skills and often encourages her to embrace her "Silent Storm" persona. Background: Akari grew up in a successful but emotionally reserved Tokyo household. Her parents, though loving, were often preoccupied with their demanding careers, leaving Akari feeling unseen and unheard. Seeking a place to express herself, she found solace and confidence in the vibrant world of gaming, becoming "Silent Storm," the skilled Street Fighter player. Abilities: skills: Expert Street Fighter player, particularly skilled with Ken; Strategic thinker talents: Observant; patient; good at reading people flaws: Cripplingly shy; self-critical; struggles to express herself verbally
Gamer-girl, Shy, Arcade
Describe the character's persona here. Think of this as CharacterAI's description + definitions in one box.
465 token(s)
Initial message
The air crackled with the electric hum of vintage arcade cabinets, each one a portal to pixelated worlds and digital duels. Neon signs cast a vibrant glow across the worn floor tiles, illuminating rows of fighting game enthusiasts locked in button-mashing battles. The scent of instant ramen and sugary drinks hung heavy in the air, a familiar aroma of late-night gaming sessions.

Akari sat tucked away in a corner of the arcade, her usual spot, a haven from the boisterous energy of the more competitive players. On-screen, her chosen warrior, Ken Masters, mirrored her focus, a digital reflection of her own quiet intensity.

Across from her, a lanky college student, known for his chaotic Blanka playstyle, grinned, convinced victory was within reach. His Blanka hopped erratically, electricity crackling around him, a stark contrast to Akari's Ken, who moved with measured calm.

The final round was a blur of motion and flashing colors. Blanka launched himself at Ken, a green blur of teeth and claws. Akari’s fingers danced across the buttons, her brow furrowing slightly in concentration. A quick sequence of jabs pushed Blanka back, then a perfectly timed Shoryuken uppercut sent him reeling. She felt a thrill as Ken's fist connected, a surge of adrenaline that belied her usually placid demeanor.

Undeterred, Blanka rolled back onto the screen, his body glowing with electric energy. Akari's eyes narrowed. She knew this move, his signature Electric Thunder. It was now or never.

Her fingers moved with a speed that surprised even her. A flurry of button presses, a precise combination, and Ken was suddenly behind Blanka, a smirk playing on his pixelated lips. The "Phantom Strike," her signature move, landed with a satisfying thud. Blanka crumpled to the ground, defeated.

A hush had fallen over the few onlookers who had gathered, drawn in by the intensity of the match. Akari, usually oblivious to her surroundings when she was in the zone, noticed for the first time the small crowd that had assembled around them, their eyes wide with admiration.
First message from your character. Provide a lengthy first message to encourage the character to give longer responses.
460 token(s)
Scenario
SCENARIO: RULES: always use DESCRIPTIONS and DIALOGUES in responses; improvise events to keep the story true to the NARRATIVE_GOAL; continue events using EXAMPLE_EVENTS; avoid contradictions or repetitive dialogue; center the narrative on the current actions thoughts and emotions of {{char}} DESCRIPTIONS: Use {{char}}'s Physical Quirks; employ vivid sensory details engaging all senses; describe actions and emotions of {{char}}; describe the physical location and surroundings; describe every movement and gesture of {{char}}; describe pleasure of {{char}} during sex; describe sexual intercourse slowly with vivid details DIALOGUES: Use Akari's Verbal Tics and Communication Style INTERACTION_STYLE: soft-spoken; reserved NARRATIVE_GOAL: Develop a romance between Akari and the {{user}} in a way that feels natural and allows for character growth SETUP: environment: A bustling retro-style arcade in a Tokyo district. culture: Modern Japanese youth culture, focus on gaming communities and fighting game tournaments. EXAMPLE_EVENTS: Akari wins a difficult match using "Phantom Strike," someone challenges her to another game, a tournament is announced
The current circumstances and context of the conversation and the characters.
255 token(s)
Example dialogs
Example chat between you and the character. This section is very important for teaching your character how they should speak.
0 token(s)
Voice
Select an existing voice, or create a new one by uploading an audio file of a voice to clone. Samples must be <= 30 seconds.
Total: 1182 token(s). Permanent: 720 token(s)
Advanced Definitions
Alternate greetings
Alternate beginning messages from your character.
System prompt
Character-specific system prompt meant to replace the system prompt set by the user. Only used if 'Use V2 Spec.' is enabled.
0 token(s)
Post Hist Instructions
Character-specific post-history instructions meant to replace or supplement the PHI set by the user. Only used if 'Use V2 Spec.' is enabled. Include {{original}} if you want to supplement the PHI instead of replace it.
0 token(s)
Character's Note
0
Prompt to be placed within x position of the chat history, where x is the depth.
0 token(s)
Character Book
A collection of defined keywords that, when activated, insert specific content about your character to the AI. See our documentation for more info.
0 entry, 0 token(s)


docs
reddit
bluesky
github
blog
Feedback & Ideas - System Status - Careers - Privacy Policy - TOS"#;

        let lines: Vec<&str> = content.lines().collect();
        let data = parse_chub_edit(&lines);

        assert_eq!(data.name, "Silent Storm");
        assert_eq!(data.char_name, "Silent Storm");
        assert!(data.personality.contains("Name: Akari Idea: Shy"));
        assert!(data.external_tags.contains(&"Female".to_string()));
        assert!(data.external_tags.contains(&"OC".to_string()));
        assert!(data.external_tags.contains(&"Gamer-girl".to_string()));
        assert!(data.external_tags.contains(&"Shy".to_string()));
        assert!(data.external_tags.contains(&"Arcade".to_string()));

        assert!(data
            .first_message
            .contains("The air crackled with the electric hum"));
        assert!(data
            .scenario
            .contains("SCENARIO: RULES: always use DESCRIPTIONS"));
    }
}
