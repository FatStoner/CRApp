#[cfg(test)]
mod tests {
    use super::super::parse_afterhour_view;

    #[test]
    fn test_parse_afterhour_alexandra() {
        let content = r#"
General
Home
Personas
Profile
Subscribe
Character
Create Character
My Characters
My Chats
Favorites
Lorebook
Create Lorebook
Lorebooks
My Lorebooks

Signout
Terms of Service
Toggle Sidebar

avatar
Edit Character
Basic Information

avatar
Import
Name
Alexandra Jones
Title
Free Use Roommate
Greeting
Sunlight streamed through the living room window...
She'd spent the morning unpacking...

She'd wandered through the apartment...

Lost in her thoughts... "Welcome home" she breathed out softly...
401
Personality
name: Alexandra Jones nickname: Alex idea: Submissive roommate...
557
Creator Notes
Version 2.

Thanks for checking out my character!
Tags
Roommate
Female
OC
Smut

Visibility

Public

Unlisted


Private
Definition Visibility

Public


Private
Advanced Configuration
Optional

Scenario
Alex moves into the User's apartment...

Model Instructions
NARRATIVE_GOAL: To establish an erotic co-living dynamic...

Lorebook
Attach Lorebooks
Add background knowledge and context for your character
0

Entry 1
Keywords
Speed Force, Flash, Reverse Flash
Description or context about this topic
Add Entry
1301
Edit Character
Edit Alexandra Jones | AfterHours
"#;

        let lines: Vec<&str> = content.lines().collect();
        let data = parse_afterhour_view(&lines);

        assert_eq!(data.name, "Alexandra Jones");
        assert_eq!(data.title, "Free Use Roommate");
        assert!(data.first_message.contains("Sunlight streamed"));
        assert!(!data.first_message.contains("401"));
        assert!(data.personality.contains("name: Alexandra Jones"));
        assert!(data.scenario.contains("Alex moves into"));
        assert!(data.scenario.contains("NARRATIVE_GOAL:"));
        assert!(data.external_tags.contains(&"Roommate".to_string()));
        assert!(data.external_tags.contains(&"Female".to_string()));
    }

    #[test]
    fn test_parse_afterhour_anya() {
        let content = r#"
avatar
Create a character

Basic Information

avatar
Import
Name
Anya
Title
Quiet monk
Greeting
The rain was coming down in sheets...

Before he could react...

One attacker stumbled back...

The woman, Anya, stood over them...
|Anya's Mana: 11/100|
382
Personality
ANYA{name: Anya. idea: A young woman trained in a mystical martial art...} VILA{name: Vila idea: A cunning...}
692
Creator Notes
Version 2.
Tags
NSFW
Adventure
Female
OC

Visibility

Public

Scenario
RULES: Only user can control...

Model Instructions
Example: Don't speak for {{user}}.

Lorebook
Attach Lorebooks
0
Create a Character | AfterHours
"#;

        let lines: Vec<&str> = content.lines().collect();
        let data = parse_afterhour_view(&lines);

        assert_eq!(data.name, "Anya");
        assert_eq!(data.title, "Quiet monk");
        assert!(data.first_message.contains("The rain was coming down"));
        assert!(data.personality.contains("ANYA{name: Anya."));
        assert!(data.scenario.contains("RULES: Only user can control"));
        assert!(data.scenario.contains("Example: Don't speak for {{user}}."));
        assert!(data.external_tags.contains(&"NSFW".to_string()));
    }
}
