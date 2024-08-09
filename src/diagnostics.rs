use std::collections::{HashMap, HashSet};
use types::MutationConfig;

pub struct Diagnostics {
    pub initial_spell_count: usize,
    initial_word_count: usize,
    initial_word_usage: HashMap<String, usize>,
    word_splits: HashMap<String, String>,
    pub final_spell_count: usize,
    pub final_word_count: usize,
}

impl Diagnostics {
    pub fn new() -> Self {
        Self {
            initial_spell_count: 0,
            initial_word_count: 0,
            initial_word_usage: Default::default(),
            word_splits: Default::default(),
            final_spell_count: 0,
            final_word_count: 0,
        }
    }

    pub fn use_initial_word(&mut self, word: &str) {
        if let Some(count) = self.initial_word_usage.get_mut(word) {
            *count += 1;
        } else {
            self.initial_word_usage.insert(word.to_string(), 1);
            self.initial_word_count += 1;
        }
    }

    pub fn set_final_word_count(&mut self, words: &HashMap<String, Vec<String>>) {
        let mut unique_words: HashSet<&str> = HashSet::new();
        for (original, mutations) in words {
            unique_words.insert(original);
            for mutation in mutations {
                unique_words.insert(mutation);
            }
        }
        self.final_word_count = unique_words.len();
    }

    pub fn procedural_split_word(&mut self, original: &str, split: &str) {
        self.word_splits.insert(original.to_string(), split.to_string());
    }

    pub fn print(&self, config: &MutationConfig) {
        println!("initial spell count: {}", self.initial_spell_count);
        println!("initial word count: {}", self.initial_word_count);
        if config.advanced_diagnostics {
            println!("\nmost used words:");
            let mut usage = self.initial_word_usage
                .iter()
                .filter(|entry| *entry.1 > 4)
                .collect::<Vec<_>>();
            usage.sort_unstable_by(|first, second| {
                second.1.cmp(first.1)
            });
            for (word, count) in usage {
                println!("- {word}: {count}")
            }

            println!("\nprocedural word splits:");
            let mut splits = self.word_splits.iter().collect::<Vec<_>>();
            splits.sort_unstable_by(|first, second| {
                first.0.cmp(second.0)
            });
            for (original, new) in splits {
                println!("- {original}: {new}")
            }
            println!()
        }
        println!("final word count: {}", self.final_word_count);
        println!("final spell count: {}", self.final_spell_count);
    }
}
