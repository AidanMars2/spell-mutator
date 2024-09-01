use std::collections::HashSet;
use std::fs;
use zspell::Dictionary;
use types::{MutationConfig, Overrides};

pub struct Spellchecker {
    dictionary: Dictionary,
    illegal_words: HashSet<String>,
    force_allow: HashSet<String>
}

impl Spellchecker {
    pub fn load(config: &MutationConfig, overrides: &Overrides) -> Spellchecker {
        let aff_content = fs::read_to_string(&config.aff_file)
            .expect("failed to load lang config");
        let dict_content = fs::read_to_string(&config.dic_file)
            .expect("failed to load dictionary");

        let dictionary = zspell::builder()
            .dict_str(&dict_content)
            .config_str(&aff_content)
            .build()
            .expect("failed to init language");

        let illegal_words = overrides.illegal_words.clone();
        let force_allow = overrides.force_allow.clone();

        Self { dictionary, illegal_words, force_allow }
    }

    pub fn check_word(&self, word: &str) -> bool {
        self.force_allow.contains(word) ||
            (word.len() > 1 &&
                word.chars().any(|char| "aeuioy".contains(char)) &&
                !self.illegal_words.contains(word) &&
                self.dictionary.check_word(word))
    }

    pub fn force_allowed(&self, word: &str) -> bool {
        self.force_allow.contains(word)
    }
}
