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
    pub fn new(lang: &crate::models::SpellcheckLanguage) -> Option<Self> {
        let prefix = lang.to_string();
        let aff_path = format!("data/dictionaries/{}.aff", prefix);
        let dic_path = format!("data/dictionaries/{}.dic", prefix);

        let aff_content = std::fs::read_to_string(&aff_path).ok()?;
        let dic_content = std::fs::read_to_string(&dic_path).ok()?;

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

    pub fn suggest(&self, word: &str) -> Vec<String> {
        self.dict
            .entry(word)
            .suggest()
            .unwrap_or_default()
            .into_iter()
            .take(5)
            .map(|s| s.to_string())
            .collect()
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
    pub fn get_ignored_words(&self) -> Vec<String> {
        if let Ok(ignored) = self.ignored_words.read() {
            let mut words: Vec<String> = ignored.iter().cloned().collect();
            words.sort();
            words
        } else {
            Vec::new()
        }
    }

    pub fn remove_word(&self, word: &str) {
        if let Ok(mut ignored) = self.ignored_words.write() {
            if ignored.remove(word) {
                // Re-write the file without the removed word
                let content: String = ignored.iter().map(|w| format!("{}\n", w)).collect();
                if let Err(e) = std::fs::write(&self.ignored_words_path, content) {
                    eprintln!("Failed to update ignored words file: {}", e);
                }
            }
        }
    }
}
