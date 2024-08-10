use std::collections::{HashMap, HashSet};
use itertools::Itertools;
use types::Overrides;
use crate::diagnostics::Diagnostics;
use crate::spellcheck::Spellchecker;

pub fn mutate_string(
    string: &str,
    spellchecker: &Spellchecker,
    words: &mut HashMap<String, Vec<String>>,
    overrides: &Overrides,
    diagnostics: &mut Diagnostics,
    initial: bool
) -> Vec<String> {
    let spell_words = string
        .split(|char: char| { !char.is_alphanumeric() && char != '\'' })
        .map(|word| process_split(word, overrides))
        .flatten()
        .collect::<Vec<_>>();

    if initial {
        for word in spell_words.iter() {
            diagnostics.use_initial_word(word)
        }
    }

    let mut result: HashSet<String> = HashSet::new();
    let mut split_mut: Vec<String> = spell_words.iter().map(|it| it.to_string()).collect();

    for index in 0..spell_words.len() {
        let word: &str = &spell_words[index];

        if !words.contains_key(word) {
            let mutated = mutate_word(word, spellchecker, overrides, diagnostics);
            words.insert(word.to_string(), mutated);
        }
        // we always have a key here, as it is added 3 lines before
        let mutated_word = words.get(word).unwrap();

        for word_mut in mutated_word {
            split_mut[index] = word_mut.clone();
            result.insert(split_mut.iter().join(" "));
        }
        split_mut[index] = spell_words[index].to_string();
    }
    let original = spell_words.iter().join(" ");
    result.remove(&original);

    result.into_iter().collect::<Vec<_>>()
}

fn mutate_word(
    word: &str,
    spellchecker: &Spellchecker,
    overrides: &Overrides,
    diagnostics: &mut Diagnostics
) -> Vec<String> {
    if !word.contains(char::is_alphanumeric) {
        println!("WARN: word without alphanumeric letters spotted in {word}");
        return vec![]
    }

    if let Some(mutations) = overrides.overrides.get(word) {
        mutations.iter().for_each(|mut_word| diagnostics.mutate_word(mut_word));
        return mutations.clone()
    }

    let mut result: HashSet<String> = HashSet::new();
    result.extend(mutate_add_char(word, spellchecker, diagnostics));
    result.extend(mutate_remove_char(word, spellchecker));
    result.extend(mutate_change_char(word, spellchecker));
    result.extend(mutate_split_word(word, spellchecker, diagnostics, overrides));
    result.remove(word);

    result.into_iter()
        .inspect(|word| diagnostics.mutate_word(word))
        .collect::<Vec<_>>()
}

fn mutate_add_char(
    word: &str,
    spellchecker: &Spellchecker,
    diagnostics: &mut Diagnostics
) -> Vec<String> {
    let mut result: Vec<String> = vec![];

    let mut chars = word.chars().collect::<Vec<_>>();
    chars.insert(0, 'a');

    let last_index = chars.len() - 1;
    for index in 0..chars.len() {
        let is_last = index >= last_index;
        for letter in 'a'..='z' {
            chars[index] = letter;
            let mutated = chars.iter().collect::<String>();
            if spellchecker.check_word(&mutated) {
                if is_last && "sy".contains(letter) && !spellchecker.force_allowed(&mutated) {
                    diagnostics.filter_word(mutated);
                    continue
                }
                result.push(mutated)
            }
        }

        if !is_last {
            chars[index] = chars[index + 1];
        }
    }

    result
}

fn mutate_change_char(
    word: &str,
    spellchecker: &Spellchecker
) -> Vec<String> {
    let mut result: Vec<String> = vec![];

    let mut chars = word.chars().collect::<Vec<_>>();

    for index in 0..chars.len() {
        let original_letter = chars[index];

        for letter in 'a'..='z' {
            if letter == original_letter {
                continue;
            }
            chars[index] = letter;
            let word = chars.iter().collect::<String>();
            if spellchecker.check_word(&word) {
                result.push(word);
            }
        }

        chars[index] = original_letter;
    }

    result
}

fn mutate_remove_char(
    word: &str,
    spellchecker: &Spellchecker
) -> Vec<String> {
    let mut result: Vec<String> = vec![];

    let chars = word.chars().collect::<Vec<_>>();
    let mut chars_mut = chars.clone();

    chars_mut.remove(0);
    for index in 0..chars_mut.len() {
        let mutated = chars_mut.iter().collect::<String>();
        if spellchecker.check_word(&mutated) {
            result.push(mutated)
        }

        chars_mut[index] = chars[index + 1]
    }

    result
}

fn mutate_split_word(
    word: &str,
    dictionary: &Spellchecker,
    diagnostics: &mut Diagnostics,
    overrides: &Overrides
) -> Vec<String> {
    let mut result: HashSet<String> = HashSet::new();

    if word.len() < 8 {
        return result.into_iter().collect()
    }

    let chars = word.chars().collect::<Vec<_>>();

    let option_skip_index = overrides.allow_split.get(word);

    // split words must be at least 3 letters long
    for index in 3..chars.len() - 3 {
        if let Some(skip_index) = option_skip_index {
            if index == *skip_index {
                continue
            }
        }

        let word_one = chars[..index].into_iter().collect::<String>();
        let word_two = chars[index..].into_iter().collect::<String>();

        if !(dictionary.check_word(&word_one) && dictionary.check_word(&word_two)) {
            continue
        }
        let split = format!("{word_one} {word_two}");
        diagnostics.procedural_split_word(word.to_string(), split.clone());
        result.insert(split);
    }

    result.into_iter().collect()
}

fn process_split(
    mut word: &str,
    overrides: &Overrides
) -> Vec<String> {
    let mut result: Vec<String> = vec![];

    loop {
        if let Some(index) = overrides.allow_split.get(word) {
            let (first, second) = word.split_at(*index);
            result.push(first.to_string());
            word = second;
        } else {
            result.push(word.to_string());
            break
        }
    }

    result
}
