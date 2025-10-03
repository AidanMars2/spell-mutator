use crate::spellchecking::{CheckResult, SpellChecker};
use std::collections::HashMap;

pub struct FreqSpellChecker {
    words: HashMap<&'static str, usize>,
    relations: Vec<(u8, Vec<usize>)>,
}

impl SpellChecker for FreqSpellChecker {
    fn name(&self) -> &'static str {
        "Frequency"
    }

    fn check(&self, original: &str, word: &str) -> CheckResult {
        if original == word {
            return CheckResult::SUCCESS;
        }
        if let Some(word_idx) = self.words.get(word) {
            let word_freq_code = self.relations[*word_idx].0.saturating_sub(18);
            if let Some(original_idx) = self.words.get(original) {
                let relations = &self.relations[*word_idx].1;
                if relations.contains(original_idx) {
                    return CheckResult::new(word_freq_code + 50);
                }
            }
            return CheckResult::new(self.relations[*word_idx].0);
        }
        CheckResult::FAIL
    }
}
