#![allow(unused)]

use crate::format::format_mutations;
use crate::mutation::{MutationContext, MutationTarget};
use crate::spellchecking::lemma::LemmaSpellChecker;
use crate::spellchecking::old::OldSpellChecker;
use crate::spellchecking::SpellChecker;
use dashmap::mapref::one::RefMut;
use dashmap::DashMap;
use humantime::format_duration;
use rayon::prelude::*;
use std::collections::HashMap;
use std::path::PathBuf;
use std::str::FromStr;
use std::sync::atomic::Ordering;
use std::time::Instant;
use std::{fs, mem};
use types::{MutationConfig, Spell, DIAGNOSTICS_FILE, MUTATED_SPELLS_JSON, MUTATED_WORDS_FILE};

mod diagnostics;
mod format;
mod mutation;
mod spellchecking;

fn main() {
    let start_time = Instant::now();
    let (config, mut spells) = parse_files();

    let lemma_target =
        MutationTarget::new(Box::new(LemmaSpellChecker::new()) as Box<dyn SpellChecker>);
    let legacy_target =
        MutationTarget::new(Box::new(OldSpellChecker::new()) as Box<dyn SpellChecker>);
    let mut ctx = MutationContext::new(config, vec![lemma_target, legacy_target]);
    let mut mutations = DashMap::new();

    spells.par_iter().for_each(|spell| {
        ctx.mutate(
            &spell
                .name
                .chars()
                .filter(|c| *c != '\'')
                .map(|c| c.to_ascii_lowercase())
                .map(|c| if !c.is_ascii_alphabetic() { ' ' } else { c })
                .collect::<String>(),
            ctx.config.mutation_depth,
        );
        for target in &ctx.targets {
            let results = target
                .results
                .remove(&spell.name)
                .map(|(_, it)| it)
                .unwrap();
            target
                .diagnostics
                .final_spell_count
                .fetch_add(results.len(), Ordering::Relaxed);
            mutations
                .entry(target.spellchecker.name())
                .or_insert_with(HashMap::new)
                .insert(spell.name.clone(), results);
        }
    });

    let mut output = PathBuf::from_str(&ctx.config.output_dir).unwrap();
    for target in &mut ctx.targets {
        println!("\n\nDiagnostics for {}:", target.spellchecker.name());
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
            fs::write(&output, serde_json::to_string(&mutations).unwrap())
                .expect("failed to write output");
            output.pop();

            output.pop();
        }
        output.pop();
    }

    format_mutations(spells, mutations, ctx.config);

    let duration = Instant::now().duration_since(start_time);
    println!("completed mutation in {}", format_duration(duration))
}

fn parse_files() -> (MutationConfig, Vec<Spell>) {
    let config: MutationConfig =
        serde_json::from_str(&fs::read_to_string("./config.json").expect("failed to load config"))
            .expect("failed to parse config");

    let mut spells: Vec<Spell> = serde_json::from_str(
        &fs::read_to_string(&config.input_file).expect("failed to load spells"),
    )
    .expect("failed to parse spells");

    spells.sort_unstable();

    (config, spells)
}
