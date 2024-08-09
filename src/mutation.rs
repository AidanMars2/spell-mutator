use std::collections::{HashMap, HashSet};
use zspell::Dictionary;
use types::Overrides;

pub fn mutate_string(
    string: &str,
    dictionary: &Dictionary,
    words: &mut HashMap<String, Vec<String>>,
    overrides: &Overrides
) -> Vec<String> {
    if let Some(words) = overrides.overrides.get(string) {
        return words.clone()
    }

    let string_split = string
        .split(|char: char| { !char.is_alphanumeric() && char != '\'' })
        .map(|word| process_split(word, overrides))
        .flatten()
        .collect::<Vec<_>>();

    let mut result: HashSet<String> = HashSet::new();
    let mut split_mut: Vec<String> = string_split.iter().map(|it| it.to_string()).collect();

    for index in 0..string_split.len() {
        let word: &str = &string_split[index];

        if !words.contains_key(word) {
            let mutated = get_mutated_words(word, dictionary, words, overrides);
            words.insert(word.to_string(), mutated);
        }
        let mutated_word = words.get(word).unwrap();

        for word_mut in mutated_word {
            split_mut[index] = word_mut.clone();
            result.insert(
                split_mut.iter()
                    .map(|word| word.trim().to_string() + " ")
                    .collect::<String>()
                    .trim()
                    .to_string()
            );
        }
        split_mut[index] = string_split[index].to_string();
    }

    result.into_iter().collect::<Vec<_>>()
}

fn get_mutated_words(
    word: &str,
    dictionary: &Dictionary,
    words: &mut HashMap<String, Vec<String>>,
    overrides: &Overrides
) -> Vec<String> {
    if !word.contains(char::is_alphanumeric) {
        return vec![]
    }

    if words.contains_key(word) {
        return words[word].clone()
    }

    if let Some(mutations) = overrides.overrides.get(word) {
        return mutations.clone()
    }

    let mut result: HashSet<String> = HashSet::new();
    result.extend(mutate_add_char(word, dictionary));
    result.extend(mutate_remove_char(word, dictionary));
    result.extend(mutate_change_char(word, dictionary));
    result.extend(mutate_split_word(word, dictionary));
    result.remove(word);

    let mut result_vec = result.into_iter().collect::<Vec<_>>();
    result_vec.shrink_to_fit();

    result_vec
}

fn mutate_add_char(
    word: &str,
    dictionary: &Dictionary
) -> Vec<String> {
    let mut result: Vec<String> = vec![];

    let mut chars = word.chars().collect::<Vec<_>>();
    chars.insert(0, 'a');

    let last_index = chars.len() - 1;
    for index in 0..chars.len() {
        for letter in 'a'..='z' {
            chars[index] = letter;
            let word = chars.iter().collect::<String>();
            if dictionary.check_word(&word) {
                result.push(word);
            }
        }

        if index < last_index {
            chars[index] = chars[index + 1];
        }
    }

    result
}

fn mutate_change_char(
    word: &str,
    dictionary: &Dictionary
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
            if dictionary.check_word(&word) {
                result.push(word);
            }
        }

        chars[index] = original_letter;
    }

    result
}

fn mutate_remove_char(
    word: &str,
    dictionary: &Dictionary
) -> Vec<String> {
    let mut result: Vec<String> = vec![];

    let chars = word.chars().collect::<Vec<_>>();
    let mut chars_mut = chars.clone();

    chars_mut.remove(0);
    for index in 0..chars_mut.len() {
        let mutated = chars_mut.iter().collect::<String>();
        if dictionary.check_word(&mutated) {
            result.push(mutated)
        }

        chars_mut[index] = chars[index + 1]
    }

    result
}

fn mutate_split_word(
    word: &str,
    dictionary: &Dictionary
) -> Vec<String> {
    let mut result: HashSet<String> = HashSet::new();

    if word.len() < 8 {
        return result.into_iter().collect()
    }

    let chars = word.chars().collect::<Vec<_>>();

    // split words must be at least 3 letters long
    for index in 3..chars.len() - 3 {
        let word_one = chars[..index].into_iter().collect::<String>();
        let word_two = chars[index..].into_iter().collect::<String>();

        if !(dictionary.check_word(&word_one) && dictionary.check_word(&word_two)) {
            continue
        }
        result.insert(format!("{word_one} {word_two}"));
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
