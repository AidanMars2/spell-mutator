use std::fs;
use types::{MutationConfig, Spell};

fn main() {
    let config: MutationConfig = serde_json::from_str(
        &fs::read_to_string("../../../assets/config.json").unwrap()
    ).unwrap();

    let input: Vec<Spell> = serde_json::from_str(
        &fs::read_to_string(config.output_file).unwrap()
    ).unwrap();

    let mut result: Vec<String> = vec![];

    for spell in input {
        if config.omit_zero_mutation_spells && spell.mutations.mutations.is_empty() {
            continue
        }
        result.push(spell.write_spell_information());
        let mut mutations = spell.mutations.mutations.into_iter().collect::<Vec<_>>();
        mutations.sort_unstable();
        for mut mutation in mutations {
            mutation.push('\n');
            result.push(mutation);
        }
    }

    fs::write(config.formatted_file, result.into_iter().collect::<String>())
        .expect("failed to format spells");
}
