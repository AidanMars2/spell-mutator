use crate::diagnostics::Diagnostics;
use crate::spellchecking::{CheckResult, SpellChecker};
use itertools::Itertools;
use std::cell::RefCell;
use std::{fs, mem};
use types::{MutationConfig, MutationResult, Overrides};

pub struct Mutator {
    pub ctx: MutationContext,
    pub config: MutationConfig,
}

impl Mutator {
    pub fn new(config: MutationConfig, spellchecker: Box<dyn SpellChecker>) -> Self {
        let overrides: Overrides = serde_json::from_str(
            &*fs::read_to_string(&config.overrides_file)
                .expect("failed to load overrides")
        ).expect("failed to parse overrides");
        Self {
            config,
            ctx: MutationContext {
                spellchecker, overrides,
                diagnostics: Diagnostics::new(),
            },
        }
    }

    pub fn mutate(&mut self, string: &str, depth: usize) -> MutationResult {
        if depth > self.config.mutation_depth {
            panic!("Requested depth exceeds allowable depth")
        }
        mutate_string(string, depth, &mut self.ctx)
    }
}


fn mutate_string(
    string: &str,
    depth: usize,
    ctx: &mut MutationContext
) -> MutationResult {
    for word in string.split(' ') {
        ctx.diagnostics.log_initial_word(word.to_string());
    }
    let mut result = MutationResult::new();
    let words = process_split(&string.chars().collect_vec(), &ctx.overrides);
    
    let mut chars = Vec::new();
    for word in words {
        if !chars.is_empty() {
            chars.push(' ');
        }
        chars.extend(word);
    }
    let string = chars.iter().collect::<String>();

    let mutation_state = vec![RefCell::new((vec![], 0)); depth + 1];
    mutation_state[0].borrow_mut().0.push(chars);
    loop {
        let (depth_idx, mutation_cell) = mutation_state.iter()
            .find_position(|mutations| mutations.borrow().0.is_empty())
            .unwrap_or((depth, &mutation_state[depth]));
        if depth_idx == 0 {
            break
        }
        let (mutations, _) = &mut *mutation_cell.borrow_mut();

        if mutations.is_empty() {
            let (lower_mutations, lower_mut_idx) =
                &mut *mutation_state[depth_idx - 1].borrow_mut();
            if *lower_mut_idx == lower_mutations.len() {
                lower_mutations.clear();
                *lower_mut_idx = 0;
                continue
            }
            let lower_chars = &lower_mutations[*lower_mut_idx];
            *lower_mut_idx += 1;
            result.add(&string, lower_chars.iter().collect::<String>(), ctx);
            process_single_mutation(lower_chars, ctx, mutations);
            if depth_idx != depth {
                continue
            }
        }
        for mutation in mutations.drain(..) {
            result.add(&string, mutation.iter().collect::<String>(), ctx);
        }
    }
    result.mutations.remove(&string);
    result.maybe_mutations.remove(&string);

    result
}

fn process_single_mutation(
    word: &Vec<char>,
    ctx: &mut MutationContext,
    target: &mut Vec<Vec<char>>
) {
    let mut split = process_split(word, &ctx.overrides);
    let split_mutated = split.iter()
        .map(|it| mutate_word(it, ctx))
        .collect_vec();

    for (index, mutated) in split_mutated.into_iter().enumerate() {
        let mut original = None;
        for word in mutated.into_iter() {
            if original.is_none() {
                original = Some(mem::replace(&mut split[index], word));
            } else {
                split[index] = word;
            }
            let mut res = vec![];
            for word in &split {
                if !res.is_empty() {
                    res.push(' ');
                }
                res.extend(word);
            }
            target.push(res);
        }
        if let Some(original) = original {
            split[index] = original;
        }
    }
}

fn mutate_word(
    word: &Vec<char>,
    ctx: &mut MutationContext,
) -> Vec<Vec<char>> {
    if !word.iter().any(|it| it.is_alphanumeric()) {
        println!("WARN: word without alphanumeric letters spotted in {}", word.iter().collect::<String>());
        return vec![]
    }

    let mut target = vec![];
    mutate_add_char(word.clone(), &mut target);
    mutate_remove_char(word, &mut target);
    mutate_change_char(word.clone(), &mut target);
    mutate_split_word(word.clone(), ctx, &mut target);
    target
}

fn mutate_add_char(
    mut chars: Vec<char>,
    target: &mut Vec<Vec<char>>
) {
    chars.insert(0, 'a');

    let last_index = chars.len() - 1;
    for index in 0..chars.len() {
        let is_last = index >= last_index;
        for letter in 'a'..='z' {
            chars[index] = letter;
            target.push(chars.clone());
        }

        if !is_last {
            chars[index] = chars[index + 1];
        }
    }
}

fn mutate_change_char(
    mut chars: Vec<char>,
    target: &mut Vec<Vec<char>>
) {
    for index in 0..chars.len() {
        let original_letter = chars[index];

        for letter in 'a'..='z' {
            if letter == original_letter {
                continue;
            }
            chars[index] = letter;
            target.push(chars.clone())
        }

        chars[index] = original_letter;
    }
}

fn mutate_remove_char(
    chars: &Vec<char>,
    target: &mut Vec<Vec<char>>
) {
    let mut chars_mut = chars.clone();
    chars_mut.remove(0);

    for index in 0..chars_mut.len() {
        let next = chars[index + 1];
        chars_mut[index] = next;
        target.push(chars_mut.clone())
    }
}

fn mutate_split_word(
    mut chars: Vec<char>,
    ctx: &mut MutationContext,
    target: &mut Vec<Vec<char>>
) {
    if chars.len() < 8 {
        return
    }
    let original = chars.clone();
    let option_skip_index = ctx.overrides.allow_split.get(&chars.iter().collect::<String>());
    chars.insert(0, ' ');

    // split words must be at least 1 letter long
    for index in 1..chars.len() - 1 {
        let next = chars[index];
        chars[index] = ' ';
        chars[index - 1] = next;
        if let Some(skip_index) = option_skip_index {
            if index == *skip_index {
                continue
            }
        }
        let string = chars.iter().collect::<String>();
        if let Some(mutations) = ctx.diagnostics
            .mutations_mut(ctx.spellchecker.check_split("", &string)) {
            mutations.log_procedural_split(original.iter().collect(), string)
        }

        target.push(chars.clone());
    }
}

fn process_split(
    string: &Vec<char>,
    overrides: &Overrides
) -> Vec<Vec<char>> {
    let mut result = vec![];
    for mut word in string.split(|it| *it == ' ') {
        while let Some(index) = overrides.allow_split.get(&word.iter().collect::<String>()) {
            let (first, second) = word.split_at(*index);
            result.push(first.to_vec());
            word = second;
        }
        result.push(word.to_vec())
    }

    result
}

pub struct MutationContext {
    pub spellchecker: Box<dyn SpellChecker>,
    pub overrides: Overrides,
    pub diagnostics: Diagnostics,
}

pub trait MutationResultExt {
    fn add(&mut self, original: &str, mutation: String, ctx: &mut MutationContext);
}

impl MutationResultExt for MutationResult {
    fn add(&mut self, original: &str, mutation: String, ctx: &mut MutationContext) {
        let check_result = ctx.spellchecker.check_split(original, &mutation);
        match check_result {
            CheckResult::Success => {
                for word in mutation.split(' ') {
                    ctx.diagnostics.results.log_mutated_word(word.to_string())
                }
                self.mutations.insert(mutation);
            }
            CheckResult::Maybe => {
                for word in mutation.split(' ') {
                    ctx.diagnostics.maybe_results.log_mutated_word(word.to_string())
                }
                self.maybe_mutations.insert(mutation);
            }
            _ => {}
        }
    }
}
