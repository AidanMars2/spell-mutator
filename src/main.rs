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
use std::sync::Arc;
use types::{MutationConfig, Spell, DIAGNOSTICS_FILE, MUTATED_SPELLS_JSON, MUTATED_WORDS_FILE};
use crate::spellchecking::freq::FreqSpellChecker;

mod diagnostics;
mod format;
mod mutation;
mod spellchecking;

fn main() {
    let start_time = Instant::now();
    let (config, mut spells) = parse_files();

    let lemma_target = MutationTarget::new(
        Box::new(LemmaSpellChecker::new()) as Box<dyn SpellChecker>);
    let freq_target = MutationTarget::new(
        Box::new(FreqSpellChecker::new()) as Box<dyn SpellChecker>);
    // let legacy_target = MutationTarget::new(
    //     Box::new(OldSpellChecker::new()) as Box<dyn SpellChecker>);
    let spell_checker_init_end_time = Instant::now();
    let mut ctx = MutationContext::new(config, vec![lemma_target, freq_target]);
    let mut mutations: DashMap<&'static str, HashMap<_, _>> = DashMap::new();
    
    for target in &mut ctx.targets {
        target.diagnostics.initial_spell_count = spells.len();
    }

    spells.par_iter().for_each(|spell| {
        let mutation_name = spell
            .name
            .chars()
            .filter(|c| *c != '\'')
            .map(|c| c.to_ascii_lowercase())
            .map(|c| if !c.is_ascii_alphabetic() { ' ' } else { c })
            .collect::<String>();
        ctx.mutate(
            &mutation_name,
            ctx.config.mutation_depth,
        );
        for target in &ctx.targets {
            if let Some(results) = target.take_mutations(&mutation_name) {
                target.diagnostics.final_spell_count.fetch_add(results.len(), Ordering::Relaxed);
                mutations.entry(target.spellchecker.name())
                    .or_default().value_mut()
                    .insert(spell.name.clone(), results);
            }
        }
        println!("completed {}", spell.name);
    });
    let mutation_end_time = Instant::now();

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
        
        output.push(MUTATED_SPELLS_JSON);
        fs::write(&output, serde_json::to_string(
            mutations.get(target.spellchecker.name()).unwrap().value()
        ).unwrap()).expect("failed to write output");
        output.pop();

        output.pop();
    }

    format_mutations(spells, mutations, ctx.config);
    let output_end_time = Instant::now();

    let dict_init_duration = spell_checker_init_end_time.duration_since(start_time);
    let mutation_duration = mutation_end_time.duration_since(spell_checker_init_end_time);
    let output_duration = output_end_time.duration_since(mutation_end_time);
    println!("\n\ndict parse time: {}\nmutation time: {}\noutput time: {}",
             format_duration(dict_init_duration),
             format_duration(mutation_duration),
             format_duration(output_duration)
    );
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
