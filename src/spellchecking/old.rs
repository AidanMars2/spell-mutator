use std::fs;
use zspell::Dictionary;
use crate::spellchecking::{CheckResult, SpellChecker};
use crate::spellchecking::CheckResult::{Fail, Maybe, Success};

pub struct OldSpellChecker {
    dictionary: Dictionary
}

impl OldSpellChecker {
    pub(crate) fn new() -> OldSpellChecker {
        let aff_content = fs::read_to_string("./dicts/lang_en_US.aff")
            .expect("failed to load lang config");
        let dict_content = fs::read_to_string("./dicts/lang_en_US_DICT.dic")
            .expect("failed to load dictionary");

        let dictionary = zspell::builder()
            .dict_str(&dict_content)
            .config_str(&aff_content)
            .build()
            .expect("failed to init language");

        Self { dictionary, }
    }
}

impl SpellChecker for OldSpellChecker {
    fn check(&self, original: &str, word: &str) -> CheckResult {
        if !self.dictionary.check_word(word) {
            return Fail
        }
        if word.len() <= 1 || !word.chars().any(|char| "aeuioy".contains(char)) {
            return Maybe
        }
        if original.len() >= 1 {
            if original[1..] == *word && original.as_bytes()[0] == b'a' {
                return Maybe
            }
            if original[..original.len() - 1] == *word &&
                "sy".contains(original.as_bytes()[original.len() - 1] as char) {
                return Maybe
            }
        }
        Success
    }
}
