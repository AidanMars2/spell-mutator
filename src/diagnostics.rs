use crate::spellchecking::CheckResult;
use dashmap::DashMap;
use itertools::Itertools;
use std::collections::{HashMap, HashSet};
use std::sync::atomic::{AtomicUsize, Ordering};
use types::MutationConfig;

pub struct Diagnostics {
    pub initial_spell_count: usize,
    initial_word_usage: DashMap<String, usize>,
    word_splits: DashMap<String, HashSet<(CheckResult, String)>>,
    pub final_spell_count: AtomicUsize,
}

impl Diagnostics {
    pub fn new() -> Self {
        Self {
            initial_spell_count: 0,
            initial_word_usage: Default::default(),
            word_splits: Default::default(),
            final_spell_count: AtomicUsize::default(),
        }
    }

    pub fn log_initial_word(&self, word: String) {
        *self.initial_word_usage.entry(word).or_insert(0) += 1;
    }

    pub fn log_procedural_split(&self, original: String, split: String, check_result: CheckResult) {
        self.word_splits
            .entry(original)
            .or_default()
            .value_mut()
            .insert((check_result, split));
    }

    pub fn stringify(&self, config: &MutationConfig, verbose: bool) -> String {
        let mut lines: Vec<String> = vec![];

        lines.push(format!("initial spell count: {}", self.initial_spell_count));
        lines.push(format!("initial word count: {}", self.initial_word_usage.len()));
        lines.push(format!(
            "final spell count: {}",
            self.final_spell_count.load(Ordering::Relaxed)
        ));
        if config.advanced_diagnostics {
            self.advanced_diagnostics(&mut lines, verbose);
        }

        lines.join("\n")
    }

    fn advanced_diagnostics(&self, lines: &mut Vec<String>, verbose: bool) {
        self.initial_word_counts(lines, verbose);

        if verbose { 
            // procedurally split words
            lines.push("\nprocedurally split words:".to_string());
            split_words(lines, &self.word_splits);
        }
    }

    fn initial_word_counts(&self, lines: &mut Vec<String>, verbose: bool) {
        // most used initial words
        lines.push("\nmost used words:".to_string());

        let mut words_by_count = HashMap::new();
        for it in self.initial_word_usage.iter() {
            words_by_count
                .entry(*it.value())
                .or_insert_with(Vec::new)
                .push(it.key().clone());
        }
        let mut words_by_count = words_by_count.into_iter().collect::<Vec<_>>();
        words_by_count.sort_unstable_by(|first, second| second.0.cmp(&first.0));

        let mut total_vertical = 0;
        for (count, words) in words_by_count {
            if total_vertical >= 10 {
                if verbose {
                    compressed_count(lines, count, words);
                    continue;
                } else {
                    break;
                }
            }
            total_vertical += words.len();
            vertical_count(lines, count, words)
        }
    }
}

fn vertical_count(lines: &mut Vec<String>, count: usize, words: Vec<String>) {
    for word in words {
        lines.push(format!("- {word}: {count}"))
    }
}

fn compressed_count(lines: &mut Vec<String>, count: usize, mut words: Vec<String>) {
    words.sort_unstable();
    lines.push(format!("\nwords used {count} times:"));
    let mut line_words: Vec<&str> = vec![];
    let mut line_len = 0;
    for word in &words {
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

fn split_words(
    lines: &mut Vec<String>,
    split: &DashMap<String, HashSet<(CheckResult, String)>>,
) {
    let mut split = split
        .iter()
        .collect_vec();
    let mut split = split.iter()
        .map(|it| it.value().iter()
            .map(|(check, split)| (check, it.key().as_str(), split.as_str())))
        .flatten()
        .collect_vec();
    split.sort_unstable();
    for (check, original, split) in split {
        let prefix = check.to_string();
        lines.push(format!("{prefix}{original}: {split}"));
    }
}

const MAX_LINE_LENGTH: usize = 80;
