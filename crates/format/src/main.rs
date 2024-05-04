use std::fs;
use types::{MutationConfig, Spell};

fn main() {
    let config: MutationConfig = serde_json::from_str(
        &fs::read_to_string("config.json").unwrap()
    ).unwrap();

    let input: Vec<Spell> = serde_json::from_str(
        &fs::read_to_string(config.output_file).unwrap()
    ).unwrap();

    let mut result: Vec<String> = vec![];

    for mut spell in input {
        result.push(spell.write_spell_information());
        spell.mutations.sort_unstable();
        for mutation in spell.mutations {
            result.push(mutation);
            result.push("\n".to_string());
        }
    }

    fs::write(config.formatted_file, result.into_iter().collect::<String>())
        .expect("failed to format spells");
}
