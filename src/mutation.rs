mod add_char;
mod change_char;
mod mutate_string;
mod remove_char;

use crate::diagnostics::Diagnostics;
use crate::mutation::mutate_string::MutateStringIter;
use crate::spellchecking::{CheckResult, SpellChecker};
use dashmap::DashMap;
use itertools::Itertools;
use std::cell::RefCell;
use std::collections::{HashMap, HashSet};
use std::mem::MaybeUninit;
use std::rc::Rc;
use std::{fs, mem};
use std::sync::Arc;
use types::{MutationConfig, Overrides};

pub struct MutationContext {
    pub overrides: Overrides,
    pub config: MutationConfig,
    pub targets: Vec<MutationTarget>,
}

impl MutationContext {
    pub fn new(config: MutationConfig, targets: Vec<MutationTarget>) -> Self {
        let overrides: Overrides = serde_json::from_str(
            &fs::read_to_string(&config.overrides_file).expect("failed to load overrides"),
        )
        .expect("failed to parse overrides");

        Self {
            config,
            overrides,
            targets,
        }
    }

    pub fn submit(&self, original: &str, processed: &str, mutation: &str, depth: usize) {
        for target in &self.targets {
            target.submit(original, processed, mutation, depth)
        }
    }

    pub fn log_initial_word(&self, word: &str) {
        for target in &self.targets {
            target.diagnostics.log_initial_word(word.to_string());
        }
    }

    pub fn mutate(&self, string: &str, depth: usize) {
        mutate_string(string, depth, self);
    }
}

pub struct MutationTarget {
    pub spellchecker: Box<dyn SpellChecker>,
    pub diagnostics: Diagnostics,
    pub results: DashMap<String, HashMap<String, (CheckResult, usize)>>,
}

impl MutationTarget {
    pub fn new(spellchecker: Box<dyn SpellChecker>) -> Self {
        Self {
            spellchecker,
            diagnostics: Diagnostics::new(),
            results: DashMap::new(),
        }
    }

    pub fn submit(&self, original: &str, processed: &str, mutation: &str, depth: usize) {
        if processed == mutation {
            return;
        }
        let check_result = processed
            .split('$')
            .zip_eq(mutation.split('$'))
            .fold(CheckResult::SUCCESS, |b, (original, mutation)| {
                self.spellchecker.check_split(original, mutation).worst(b)
            });

        if !check_result.is_fail() {
            let result = String::new();

            for (original, split) in processed.split('$').zip(mutation.split('$')) {
                if split.contains(' ') {
                    self.diagnostics.log_procedural_split(
                        original.to_string(),
                        split.to_string(),
                        check_result,
                    )
                }
            }
            self.results
                .entry(original.to_string())
                .or_default()
                .value_mut()
                .insert(mutation.split('$').join(" "), (check_result, depth));
        }
    }

    pub fn take_mutations(&self, original: &str) -> Option<HashMap<String, (CheckResult, usize)>> {
        self.results.remove(original).map(|(_, it)| it)
    }
}

fn mutate_string(string: &str, depth: usize, ctx: &MutationContext) {
    let words = process_split(string.as_bytes(), &ctx.overrides);
    for word in &words {
        ctx.log_initial_word(str::from_utf8(word).unwrap());
    }

    let mut chars = Vec::new();
    for word in words {
        if !chars.is_empty() {
            chars.push(b'$');
        }
        chars.extend(word);
    }
    let processed_string = String::from_utf8(chars.clone()).unwrap();

    let mut mutation_state = vec![];
    for _ in 0..depth {
        mutation_state.push(RefCell::new(None))
    }
    mutation_state[0] = RefCell::new(Some(MutateStringIter::new(&chars)));
    loop {
        let (depth_idx, mutation_iter) = mutation_state
            .iter()
            .find_position(|mutations| mutations.borrow().is_none())
            .unwrap_or_else(|| (0, &mutation_state[0]));

        if mutation_iter.borrow().is_none() {
            if depth_idx == 0 {
                break;
            }
            let lower_mutation_iter = &mutation_state[depth_idx - 1];
            if let Some(mutation) = lower_mutation_iter.borrow_mut().as_mut().unwrap().next() {
                ctx.submit(
                    string,
                    &processed_string,
                    str::from_utf8(mutation).unwrap(),
                    depth_idx + 1,
                );
                *mutation_iter.borrow_mut() = Some(MutateStringIter::new(mutation));
                if depth_idx != depth - 1 {
                    continue;
                }
            } else {
                *lower_mutation_iter.borrow_mut() = None;
                continue;
            }
        }

        let mut mutation_iter_option = mutation_iter.borrow_mut();
        let mut mutation_iter = mutation_iter_option.as_mut().unwrap();
        while let Some(mutation) = mutation_iter.next() {
            ctx.submit(
                string,
                &processed_string,
                str::from_utf8(mutation).unwrap(),
                depth,
            )
        }
        *mutation_iter_option = None
    }
}

fn process_split<'a>(string: &'a [u8], overrides: &Overrides) -> Vec<&'a [u8]> {
    for letter in string {
        if !(letter.is_ascii_lowercase() || *letter == b' ') {
            panic!(
                "unknown character spotted in \"{}\"",
                str::from_utf8(string).unwrap()
            )
        }
    }

    let mut result = vec![];
    for mut word in string.split(|it| *it == b' ') {
        while let Some(index) = overrides.allow_split.get(str::from_utf8(word).unwrap()) {
            let (first, second) = word.split_at(*index);
            result.push(first);
            word = second;
        }
        result.push(word)
    }

    result
}
