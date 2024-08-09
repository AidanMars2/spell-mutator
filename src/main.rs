use std::collections::{HashMap, HashSet};
use std::fs;
use zspell::Dictionary;
use types::{MutationConfig, Overrides, Spell};
use crate::mutation::mutate_string;

mod mutation;

fn main() {
    let (config, mut spells, overrides) = parse_files();

    let initial_spells = spells.len();
    println!("initial spell count: {initial_spells}");

    let dict = create_dictionary();
    let mut words_mut: HashMap<String, Vec<String>> = HashMap::new();

    let mut index = 0;
    spells.iter_mut().for_each(|spell| {
        index += 1;
        println!("mutating {index}/{initial_spells}: {}", spell.name);
        mutate_spell(spell, &mut words_mut, &dict, &overrides, config.mutation_depth);
    });

    println!("final spell count: {}",
             spells
                 .iter()
                 .map(|spell| spell.mutations.len())
                 .sum::<usize>()
    );

    fs::write(config.output_file, serde_json::to_string(&spells).unwrap())
        .expect("failed to write output");
}

fn mutate_spell(
    spell: &mut Spell,
    words_mut: &mut HashMap<String, Vec<String>>,
    dictionary: &Dictionary,
    overrides: &Overrides,
    mutation_depth: u8
) {
    let mut result: HashSet<String> = HashSet::new();
    let mut current: Vec<String> = vec![spell.name.to_lowercase()];
    let mut next: HashSet<String> = HashSet::new();

    for _ in 0..mutation_depth {
        current.drain(..).for_each(|name| {
            let mutations = mutate_string(&name, dictionary, words_mut, overrides);
            mutations.into_iter().for_each(|mutation| {
                next.insert(mutation.clone());
                result.insert(mutation);
            })
        });
        current.extend(next.drain());
    }

    spell.mutations.extend(result);
}

fn create_dictionary() -> Dictionary {
    let aff_content = fs::read_to_string("lang_en_US.aff")
        .expect("failed to load lang config");
    let dict_content = fs::read_to_string("lang_en_US_DICT.dic")
        .expect("failed to load dictionary");

    zspell::builder()
        .dict_str(&dict_content)
        .config_str(&aff_content)
        .build()
        .expect("failed to init language")
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
