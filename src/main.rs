#![allow(unused)]

use std::collections::HashMap;
use crate::spellchecking::SpellChecker;
use humantime::format_duration;
use std::{fs, mem};
use std::path::PathBuf;
use std::str::FromStr;
use std::time::Instant;
use types::{MutationConfig, MutationResult, Spell, DIAGNOSTICS_FILE, MUTATED_SPELLS_JSON, MUTATED_WORDS_FILE};
use crate::mutation::{MutationContext, MutationTarget};
use crate::spellchecking::lemma::LemmaSpellChecker;
use crate::spellchecking::old::OldSpellChecker;

mod mutation;
mod diagnostics;
mod spellchecking;

fn main() {
    let start_time = Instant::now();
    let (config, mut spells) = parse_files();

    let lemma_target = MutationTarget::new(Box::new(LemmaSpellChecker::new()) as Box<dyn SpellChecker>);
    let legacy_target = MutationTarget::new(Box::new(OldSpellChecker::new()) as Box<dyn SpellChecker>);
    let mut ctx = MutationContext::new(config, vec![lemma_target, legacy_target]);
    let mut mutations = HashMap::new();

    for spell in &mut spells {
        spell.name.make_ascii_lowercase();
        spell.name.retain(|it| it != '\'');
        ctx.mutate(&spell.name.replace('-', " "), ctx.config.mutation_depth);
        for target in &mut ctx.targets {
            let results = mem::replace(&mut target.results, MutationResult::new());
            target.diagnostics.results.final_spell_count += results.mutations.len();
            target.diagnostics.maybe_results.final_spell_count += results.maybe_mutations.len();
            mutations
                .entry(target.spellchecker.name())
                .or_insert_with(HashMap::new)
                .insert(spell.name.clone(), results);
        }
    }

    let mut output = PathBuf::from_str(&*ctx.config.output_dir).unwrap();
    for target in &mut ctx.targets {
        println!("Diagnostics for {}:", target.spellchecker.name());
        println!("{}", target.diagnostics.stringify(&ctx.config, false));
        output.push(target.spellchecker.name());
        fs::create_dir(&output);

        output.push(DIAGNOSTICS_FILE);
        fs::write(&output, target.diagnostics.stringify(&ctx.config, true))
            .expect("failed to write diagnostics");
        output.pop();

        for depth in 1..=ctx.config.mutation_depth {
            output.push(format!("{depth}deep"));
            fs::create_dir(&output);

            output.push(MUTATED_SPELLS_JSON);
            fs::write(&output, serde_json::to_string(&spells).unwrap())
                .expect("failed to write output");
            output.pop();

            output.push(MUTATED_WORDS_FILE);
            fs::write(&output, target.diagnostics.mutated_words())
                .expect("failed to write mutated words");
            output.pop();

            output.pop();
        }
        output.pop();
    }

    let duration = Instant::now().duration_since(start_time);
    println!("completed mutation in {}", format_duration(duration))
}

fn parse_files() -> (MutationConfig, Vec<Spell>) {
    let config: MutationConfig = serde_json::from_str(
        &*fs::read_to_string("./config.json")
            .expect("failed to load config")
    ).expect("failed to parse config");

    let mut spells: Vec<Spell> = serde_json::from_str(
        &*fs::read_to_string(&config.input_file)
            .expect("failed to load spells")
    ).expect("failed to parse spells");

    spells.sort_unstable();

    (config, spells)
}
