use std::collections::{HashMap, HashSet};
use types::MutationConfig;

pub struct Diagnostics {
    pub initial_spell_count: usize,
    initial_word_count: usize,
    initial_word_usage: HashMap<String, usize>,
    word_splits: HashMap<String, HashSet<String>>,
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

    pub fn procedural_split_word(&mut self, original: String, split: String) {
        if let Some(split_words) = self.word_splits.get_mut(&original) {
            split_words.insert(split);
        } else {
            let mut set = HashSet::<String>::new();
            set.insert(split);
            self.word_splits.insert(original.to_string(), set);
        }
    }

    pub fn stringify(&self, config: &MutationConfig, verbose: bool) -> String {
        let mut lines: Vec<String> = vec![];

        lines.push(format!("initial spell count: {}", self.initial_spell_count));
        lines.push(format!("initial word count: {}", self.initial_word_count));
        if config.advanced_diagnostics {
            self.advanced_diagnostics(&mut lines, verbose);
        }
        lines.push(format!("final word count: {}", self.final_word_count));
        lines.push(format!("final spell count: {}", self.final_spell_count));

        lines.join("\n")
    }

    fn advanced_diagnostics(&self, lines: &mut Vec<String>, verbose: bool) {
        let mut words_by_count = HashMap::<usize, Vec<String>>::new();
        for (word, count) in self.initial_word_usage.iter() {
            if let Some(in_count) = words_by_count.get_mut(&count) {
                in_count.push(word.clone())
            } else {
                words_by_count.insert(*count, vec![word.clone()]);
            }
        }
        let mut words_by_count = words_by_count.into_iter().collect::<Vec<_>>();
        words_by_count.sort_unstable_by(|first, second|
            second.0.cmp(&first.0));

        lines.push("\nmost used words:".to_string());

        let mut total_vertical = 0;
        for (count, words) in words_by_count {
            if total_vertical >= 10 {
                if verbose {
                    compressed_count(lines, count, words)
                }
                continue
            }
            total_vertical += words.len();
            vertical_count(lines, count, words)
        }

        lines.push("\nprocedurally split words:".to_string());
        for (original, split) in self.word_splits.iter() {
            split_words(lines, original, split)
        }
    }
}

fn vertical_count(lines: &mut Vec<String>, count: usize, words: Vec<String>) {
    for word in words {
        lines.push(format!("- {word}: {count}"))
    }
}

fn compressed_count(lines: &mut Vec<String>, count: usize, words: Vec<String>) {
    const MAX_LINE_LENGTH: usize = 80;
    lines.push(format!("words used {count} times:"));
    let mut line_words: Vec<String> = vec![];
    let mut line_len = 0;
    for word in words {
        line_len += word.len();
        line_len += 2;
        if line_len >= MAX_LINE_LENGTH {
            lines.push(format!("- {},", line_words.join(", ")));
            line_words.clear();
            line_len = 0;
        }
        line_words.push(word);
    }
    if !line_words.is_empty() {
        lines.push(format!("- {}", line_words.join(", ")));
    }
}

fn split_words(lines: &mut Vec<String>, original: &str, split: &HashSet<String>) {
    for split_word in split {
        lines.push(format!("- {original}: {split_word}"))
    }
}
