use zspell::Dictionary;

pub struct SpellChecker {
    dict: Dictionary,
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

        Some(Self { dict })
    }

    pub fn check(&self, text: &str) -> Vec<(usize, usize)> {
        self.dict
            .check_indices(text)
            .map(|(offset, word)| (offset, offset + word.len()))
            .collect()
    }
}
