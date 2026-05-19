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
        let dictionary_dir = Self::find_dictionary_dir()?;
        let aff_path = dictionary_dir.join("en_US.aff");
        let dic_path = dictionary_dir.join("en_US.dic");

        let aff_content = std::fs::read_to_string(&aff_path).ok()?;
        let dic_content = std::fs::read_to_string(&dic_path).ok()?;

        let dict = zspell::builder()
            .config_str(&aff_content)
            .dict_str(&dic_content)
            .build()
            .ok()?;

        let ignored_words_path = dictionary_dir
            .join("user_ignored.txt")
            .to_string_lossy()
            .to_string();
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


    fn find_dictionary_dir() -> Option<std::path::PathBuf> {
        let relative_dictionary_dir = std::path::PathBuf::from("data").join("dictionaries");

        let mut candidates = Vec::new();

        if let Ok(current_dir) = std::env::current_dir() {
            candidates.push(current_dir.join(&relative_dictionary_dir));
        }

        if let Ok(exe_path) = std::env::current_exe() {
            if let Some(exe_dir) = exe_path.parent() {
                candidates.push(exe_dir.join(&relative_dictionary_dir));
                candidates.push(exe_dir.join("..").join(&relative_dictionary_dir));
                candidates.push(exe_dir.join("..").join("..").join(&relative_dictionary_dir));
            }
        }

        candidates.push(std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(&relative_dictionary_dir));

        candidates.into_iter().find(|dir| {
            dir.join("en_US.aff").is_file() && dir.join("en_US.dic").is_file()
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
