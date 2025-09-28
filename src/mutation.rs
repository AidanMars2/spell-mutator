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
    word: &str,
    depth: usize,
    ctx: &mut MutationContext
) -> MutationResult {
    ctx.diagnostics.log_initial_word(word.to_string());
    let mut result = MutationResult::new();

    let mutation_state = vec![RefCell::new((vec![], 0)); depth + 1];
    mutation_state[0].borrow_mut().0.push(word.chars().collect_vec());
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
            result.add(word, lower_chars.iter().collect::<String>(), ctx);
            process_single_mutation(lower_chars, &ctx.overrides, mutations);
            if depth_idx != depth {
                continue
            }
        }
        for mutation in mutations.drain(..) {
            result.add(word, mutation.iter().collect::<String>(), ctx);
        }
    }

    result
}

fn process_single_mutation(
    word: &Vec<char>,
    overrides: &Overrides,
    target: &mut Vec<Vec<char>>
) {
    let mut split = process_split(&word.iter().collect::<String>(), overrides);
    let split_mutated = split.iter()
        .map(|it| mutate_word(it, overrides))
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
    overrides: &Overrides
) -> Vec<Vec<char>> {
    if !word.iter().any(|it| it.is_alphanumeric()) {
        println!("WARN: word without alphanumeric letters spotted in {}", word.iter().collect::<String>());
        return vec![]
    }

    let mut target = vec![];
    mutate_add_char(word.clone(), &mut target);
    mutate_remove_char(word, &mut target);
    mutate_change_char(word.clone(), &mut target);
    mutate_split_word(word.clone(), overrides, &mut target);
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
    overrides: &Overrides,
    target: &mut Vec<Vec<char>>
) {
    if chars.len() < 8 {
        return
    }
    let option_skip_index = overrides.allow_split.get(&chars.iter().collect::<String>());
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

        target.push(chars.clone());
    }
}

fn process_split(
    string: &str,
    overrides: &Overrides
) -> Vec<Vec<char>> {
    let mut result = vec![];
    for mut word in string.split(' ') {
        while let Some(index) = overrides.allow_split.get(word) {
            let (first, second) = string.split_at(*index);
            result.push(first.chars().collect_vec());
            word = second;
        }
        result.push(word.chars().collect_vec())
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
            CheckResult::Succes => {
                for word in mutation.split(' ') {
                    ctx.diagnostics.mutations.log_mutated_word(word.to_string())
                }
                self.mutations.insert(mutation);
            }
            CheckResult::Maybe => {
                for word in mutation.split(' ') {
                    ctx.diagnostics.maybe_mutations.log_mutated_word(word.to_string())
                }
                self.maybe_mutations.insert(mutation);
            }
            _ => {}
        }
    }
}
