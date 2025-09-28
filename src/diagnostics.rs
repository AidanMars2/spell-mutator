use std::collections::{HashMap, HashSet};
use types::MutationConfig;
use itertools::Itertools;
use crate::spellchecking::CheckResult;

pub struct Diagnostics {
    pub initial_spell_count: usize,
    initial_word_count: usize,
    initial_word_usage: HashMap<String, usize>,
    pub results: Mutations,
    pub maybe_results: Mutations
}

#[derive(Default)]
pub struct Mutations {
    word_splits: HashMap<String, HashSet<String>>,
    mutated_words: HashSet<String>,
    pub final_spell_count: usize,
}

impl Mutations {
    pub fn log_procedural_split(&mut self, original: String, split: String) {
        self.word_splits.entry(original).or_insert_with(HashSet::new).insert(split);
    }

    pub fn log_mutated_word(&mut self, word: String) {
        self.mutated_words.insert(word);
    }
}

impl Diagnostics {
    pub fn new() -> Self {
        Self {
            initial_spell_count: 0,
            initial_word_count: 0,
            initial_word_usage: Default::default(),
            results: Default::default(),
            maybe_results: Default::default(),
        }
    }
    
    pub fn mutations_mut(&mut self, check_result: CheckResult) -> Option<&mut Mutations> {
        match check_result {
            CheckResult::Success => Some(&mut self.results),
            CheckResult::Maybe => Some(&mut self.maybe_results),
            CheckResult::Fail => None
        }
    }

    pub fn log_initial_word(&mut self, word: String) {
        *self.initial_word_usage.entry(word).or_insert(0) += 1;
    }

    pub fn stringify(&self, config: &MutationConfig, verbose: bool) -> String {
        let mut lines: Vec<String> = vec![];

        lines.push(format!("initial spell count: {}", self.initial_spell_count));
        lines.push(format!("initial word count: {}", self.initial_word_count));
        lines.push(format!("final word count: {}", self.results.mutated_words.len()));
        lines.push(format!("final spell count: {}", self.results.final_spell_count));
        lines.push(format!("related word count: {}", self.maybe_results.mutated_words.len()));
        lines.push(format!("related spell count: {}", self.maybe_results.final_spell_count));
        if config.advanced_diagnostics {
            self.advanced_diagnostics(&mut lines, verbose);
        }

        lines.join("\n")
    }

    fn advanced_diagnostics(&self, lines: &mut Vec<String>, verbose: bool) {
        // most used initial words
        lines.push("\nmost used words:".to_string());

        let mut words_by_count = HashMap::<usize, Vec<&str>>::new();
        for (word, count) in self.initial_word_usage.iter() {
            if let Some(in_count) = words_by_count.get_mut(&count) {
                in_count.push(word)
            } else {
                words_by_count.insert(*count, vec![word]);
            }
        }
        let mut words_by_count = words_by_count.into_iter().collect::<Vec<_>>();
        words_by_count.sort_unstable_by(|first, second|
            second.0.cmp(&first.0));


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

        // procedurally split words
        lines.push("\nprocedurally split words:".to_string());
        for (original, split) in self.results.word_splits.iter() {
            split_words(lines, original, split);
        }
        lines.push("\nrelated split words:".to_string());
        for (original, split) in self.maybe_results.word_splits.iter() {
            split_words(lines, original, split);
        }
    }

    pub fn mutated_words(&self) -> String {
        let mut lines: Vec<String> = vec![];

        lines.push("mutated words:".to_string());
        word_listing(&mut lines, &self.results.mutated_words);
        lines.push("\nrelated words:".to_string());
        word_listing(&mut lines, &self.maybe_results.mutated_words);

        lines.join("\n")
    }
}

fn vertical_count(lines: &mut Vec<String>, count: usize, words: Vec<&str>) {
    for word in words {
        lines.push(format!("- {word}: {count}"))
    }
}

fn compressed_count(lines: &mut Vec<String>, count: usize, mut words: Vec<&str>) {
    words.sort_unstable();
    lines.push(format!("\nwords used {count} times:"));
    let mut line_words: Vec<&str> = vec![];
    let mut line_len = 0;
    for word in words {
        line_len += word.len();
        line_len += 2;
        if line_len >= MAX_LINE_LENGTH {
            lines.push(format!("- {},", line_words.drain(..).join(", ")));
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

fn word_listing(lines: &mut Vec<String>, words: &HashSet<String>) {
    let mut words = words.iter()
        .map(String::as_str)
        .collect::<Vec<_>>();
    words.sort_unstable();

    let mut line_words: Vec<&str> = vec![];
    let mut line_len = 0;
    for word in words {
        line_len += word.len();
        line_len += 2;
        if line_len >= MAX_LINE_LENGTH {
            lines.push(format!("{},", line_words.drain(..).join(", ")));
            line_len = 0;
        }
        line_words.push(word);
    }
    if !line_words.is_empty() {
        lines.push(format!("{}", line_words.join(", ")));
    }
}

const MAX_LINE_LENGTH: usize = 80;
