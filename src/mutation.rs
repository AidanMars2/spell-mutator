mod add_char;
mod change_char;
mod remove_char;
mod mutate_string;

use crate::diagnostics::Diagnostics;
use crate::spellchecking::{CheckResult, SpellChecker};
use itertools::Itertools;
use std::cell::RefCell;
use std::{fs, mem};
use std::collections::{HashMap, HashSet};
use std::mem::MaybeUninit;
use std::rc::Rc;
use types::{MutationConfig, MutationResult, Overrides};
use crate::mutation::mutate_string::MutateStringIter;

pub struct MutationContext {
    pub overrides: Overrides,
    pub config: MutationConfig,
    pub targets: Vec<MutationTarget>
}

impl MutationContext {
    pub fn new(config: MutationConfig, targets: Vec<MutationTarget>) -> Self {
        let overrides: Overrides = serde_json::from_str(
            &*fs::read_to_string(&config.overrides_file)
                .expect("failed to load overrides")
        ).expect("failed to parse overrides");

        Self { config, overrides, targets }
    }

    pub fn submit(&mut self, original: &str, mutation: &str, depth: usize) {
        for target in &mut self.targets {
            target.submit(original, mutation, depth)
        }
    }

    pub fn log_initial_word(&mut self, word: &str) {
        for target in &mut self.targets {
            target.diagnostics.log_initial_word(word.to_string());
        }
    }
    
    pub fn mutate(&mut self, string: &str, depth: usize) {
        mutate_string(string, depth, self);
    }
}

pub struct MutationTarget {
    pub spellchecker: Box<dyn SpellChecker>,
    pub diagnostics: Diagnostics,
    pub results: MutationResult
}

impl MutationTarget {
    pub fn new(spellchecker: Box<dyn SpellChecker>) -> Self {
        Self {
            spellchecker,
            diagnostics: Diagnostics::new(),
            results: MutationResult::new()
        }
    }

    pub fn submit(&mut self, original: &str, mutation: &str, depth: usize) {
        let check_result = original.split(' ')
            .zip_eq(mutation.split('$'))
            .fold(CheckResult::Success, |b, (original, mutation)| {
                self.spellchecker.check_split(original, mutation).min(b)
            });

        if let Some(target) = self.results.target_mut(check_result) {
            let result = String::new();

            let diag_target = self.diagnostics.mutations_mut(check_result).unwrap();
            for (original, split) in original.split(' ').zip(mutation.split('$')) {
                if split.contains(' ') {
                    diag_target.log_procedural_split(original.to_string(), split.to_string())
                }
            }

            let min_depth = target.entry(mutation.split([' ', '$'])
                .inspect(|word| diag_target
                    .log_mutated_word(word.to_string(), depth)).join(" "))
                .or_insert(depth);
            if depth < *min_depth {
                *min_depth = depth;
            }
        }
    }
}

fn mutate_string(
    string: &str,
    depth: usize,
    ctx: &mut MutationContext
) {
    let words = process_split(&string.as_bytes(), &ctx.overrides);
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

    let mut mutation_state = vec![];
    for _ in 0..depth {
        mutation_state.push(RefCell::new(None))
    }
    mutation_state[0] = RefCell::new(Some(MutateStringIter::new(&chars)));
    loop {
        let (depth_idx, mutation_iter) = mutation_state.iter()
            .find_position(|mutations| mutations.borrow().is_none())
            .unwrap();

        if mutation_iter.borrow().is_none() {
            if depth_idx == 0 {
                break
            }
            let lower_mutation_iter = &mutation_state[depth_idx - 1];
            if let Some(mutation) = lower_mutation_iter.borrow_mut().as_mut().unwrap().next() {
                ctx.submit(string, str::from_utf8(mutation).unwrap(), depth_idx + 1);
                *mutation_iter.borrow_mut() = Some(MutateStringIter::new(mutation));
                if depth_idx != depth - 1 {
                    continue
                }
            } else {
                *lower_mutation_iter.borrow_mut() = None;
                continue
            }
        }

        let mut mutation_iter_option = mutation_iter.borrow_mut();
        let mut mutation_iter = mutation_iter_option.as_mut().unwrap();
        while let Some(mutation) = mutation_iter.next() {
            ctx.submit(string, str::from_utf8(mutation).unwrap(), depth)
        }
        *mutation_iter_option = None
    }
}

fn process_split<'a>(
    string: &'a [u8],
    overrides: &Overrides
) -> Vec<&'a [u8]> {
    for letter in string {
        if !((b'a'..=b'z').contains(letter) || *letter == b' ') {
            panic!("unknown character spotted in \"{}\"", str::from_utf8(string).unwrap())
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

pub trait MutationResultExt {
    fn target_mut(&mut self, check_result: CheckResult) -> Option<&mut HashMap<String, usize>>;
}

impl MutationResultExt for MutationResult {
    fn target_mut(&mut self, check_result: CheckResult) -> Option<&mut HashMap<String, usize>> {
        match check_result {
            CheckResult::Success => Some(&mut self.mutations),
            CheckResult::Maybe => Some(&mut self.maybe_mutations),
            CheckResult::Fail => None
        }
    }
}
