use std::collections::{HashMap, HashSet};
use std::fs;
use types::{MutationConfig, Overrides, Spell};
use crate::diagnostics::Diagnostics;
use crate::mutation::mutate_string;
use crate::spellcheck::Spellchecker;

mod mutation;
mod diagnostics;
mod spellcheck;

fn main() {
    let (config, mut spells, overrides) = parse_files();

    let mut diagnostics = Diagnostics::new();
    diagnostics.initial_spell_count = spells.len();

    let spellchecker = Spellchecker::load(&config, &overrides);
    let mut words_mut: HashMap<String, Vec<String>> = HashMap::new();

    spells.iter_mut().for_each(|spell| {
        mutate_spell(
            spell, &mut words_mut, &spellchecker,
            &overrides, config.mutation_depth, &mut diagnostics
        );
    });
    diagnostics.final_spell_count = spells.iter().map(|spell| spell.mutations.len()).sum();
    diagnostics.set_final_word_count();
    println!("{}", diagnostics.stringify(&config, false));

    fs::write(&config.output_file, serde_json::to_string(&spells).unwrap())
        .expect("failed to write output");

    fs::write(&config.diagnostics_file, diagnostics.stringify(&config, true))
        .expect("failed to write diagnostics");

    fs::write(&config.mutated_words_file, diagnostics.mutated_words())
        .expect("failed to write mutated words");
}

fn mutate_spell(
    spell: &mut Spell,
    words_mut: &mut HashMap<String, Vec<String>>,
    spellchecker: &Spellchecker,
    overrides: &Overrides,
    mutation_depth: u8,
    diagnostics: &mut Diagnostics
) {
    let mut result: HashSet<String> = HashSet::new();
    let mut current: Vec<String> = vec![spell.name.to_lowercase()];
    let mut next: HashSet<String> = HashSet::new();

    for index in 0..mutation_depth {
        let initial = index == 0;
        current.drain(..).for_each(|name| {
            let mutations = mutate_string(
                &name, spellchecker, words_mut,
                overrides, diagnostics, initial
            );
            mutations.into_iter().for_each(|mutation| {
                next.insert(mutation.clone());
                result.insert(mutation);
            })
        });
        current.extend(next.drain());
    }

    spell.mutations.extend(result);
}

fn parse_files() -> (MutationConfig, Vec<Spell>, Overrides) {
    let config: MutationConfig = serde_json::from_str(
        &*fs::read_to_string("config.json")
            .expect("failed to load config")
    ).expect("failed to parse config");

    let mut spells: Vec<Spell> = serde_json::from_str(
        &*fs::read_to_string(&config.input_file)
            .expect("failed to load spells")
    ).expect("failed to parse spells");

    spells.sort_unstable();


    let overrides: Overrides = serde_json::from_str(
        &*fs::read_to_string(&config.overrides_file)
            .expect("failed to load overrides")
    ).expect("failed to parse overrides");

    (config, spells, overrides)
}
