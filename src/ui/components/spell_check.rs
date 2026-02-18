use std::collections::HashSet;
use std::io::Write;
use std::sync::RwLock;
use zspell::Dictionary;

pub struct SpellChecker {
    dict: Dictionary,
    ignored_words: RwLock<HashSet<String>>,
    ignored_words_path: String,
}

impl SpellChecker {
    pub fn new() -> Option<Self> {
        let aff_content = std::fs::read_to_string("data/dictionaries/en_US.aff").ok()?;
        let dic_content = std::fs::read_to_string("data/dictionaries/en_US.dic").ok()?;

        let dict = zspell::builder()
            .config_str(&aff_content)
            .dict_str(&dic_content)
            .build()
            .ok()?;

        let ignored_words_path = "data/dictionaries/user_ignored.txt".to_string();
        let mut ignored_words = HashSet::new();

        if let Ok(content) = std::fs::read_to_string(&ignored_words_path) {
            for line in content.lines() {
                if !line.trim().is_empty() {
                    ignored_words.insert(line.trim().to_string());
                }
            }
        }

        Some(Self {
            dict,
            ignored_words: RwLock::new(ignored_words),
            ignored_words_path,
        })
    }

    pub fn check(&self, text: &str) -> Vec<(usize, usize)> {
        let glitches = self.dict.check_indices(text);

        if let Ok(ignored) = self.ignored_words.read() {
            glitches
                .filter(|(_, word)| !ignored.contains(*word))
                .map(|(offset, word)| (offset, offset + word.len()))
                .collect()
        } else {
            glitches
                .map(|(offset, word)| (offset, offset + word.len()))
                .collect()
        }
    }

    pub fn add_word(&self, word: &str) {
        if let Ok(mut ignored) = self.ignored_words.write() {
            if ignored.insert(word.to_string()) {
                if let Ok(mut file) = std::fs::OpenOptions::new()
                    .create(true)
                    .append(true)
                    .open(&self.ignored_words_path)
                {
                    if let Err(e) = writeln!(file, "{}", word) {
                        eprintln!("Failed to write to ignored words file: {}", e);
                    }
                } else {
                    eprintln!("Failed to open ignored words file for appending");
                }
            }
        }
    }
}
