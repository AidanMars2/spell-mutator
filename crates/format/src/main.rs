use std::fs;
use std::path::PathBuf;
use std::str::FromStr;
use types::{MutationConfig, Spell, MUTATED_SPELLS_FILE, MUTATED_SPELLS_JSON};

fn main() {
    let config: MutationConfig = serde_json::from_str(
        &fs::read_to_string("config.json").unwrap()
    ).unwrap();

    let mut output_file = PathBuf::from_str(&*config.output_dir).unwrap();
    output_file.push(format!("{}deep", config.mutation_depth));
    output_file.push(MUTATED_SPELLS_JSON);
    let input: Vec<Spell> = serde_json::from_str(
        &fs::read_to_string(&output_file).unwrap()
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
        let mut mutations = spell.mutations.maybe_mutations.into_iter().collect::<Vec<_>>();
        mutations.sort_unstable();
        for mut mutation in mutations {
            result.push("#".to_string());
            mutation.push('\n');
            result.push(mutation);
        }
    }

    fs::write(
        output_file.with_file_name(MUTATED_SPELLS_FILE),
        result.into_iter().collect::<String>()
    ).expect("failed to format spells");
}
