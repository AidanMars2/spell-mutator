use crate::mutation::Mutator;
use crate::spellchecking::lemma::LemmaSpellChecker;
use crate::spellchecking::SpellChecker;
use humantime::format_duration;
use std::fs;
use std::time::Instant;
use types::{MutationConfig, Spell};

mod mutation;
mod diagnostics;
mod spellchecking;

fn main() {
    let start_time = Instant::now();
    let (config, mut spells) = parse_files();

    let spellchecker = Box::new(LemmaSpellChecker::new()) as Box<dyn SpellChecker>;
    let mut mutator = Mutator::new(config, spellchecker);

    let mut total_mutations = 0usize;
    let mut total_maybe_mutations = 0usize;
    for spell in &mut spells {
        spell.mutations = mutator.mutate(&spell.name, mutator.config.mutation_depth);
        total_mutations += spell.mutations.mutations.len();
        total_maybe_mutations += spell.mutations.maybe_mutations.len();
    }
    mutator.ctx.diagnostics.mutations.final_spell_count = total_mutations;
    mutator.ctx.diagnostics.maybe_mutations.final_spell_count = total_maybe_mutations;
    println!("{}", mutator.ctx.diagnostics.stringify(&mutator.config, false));

    fs::write(&mutator.config.output_file, serde_json::to_string(&spells).unwrap())
        .expect("failed to write output");

    fs::write(&mutator.config.diagnostics_file, 
              mutator.ctx.diagnostics.stringify(&mutator.config, true))
        .expect("failed to write diagnostics");

    fs::write(&mutator.config.mutated_words_file, mutator.ctx.diagnostics.mutated_words())
        .expect("failed to write mutated words");
    
    let duration = Instant::now().duration_since(start_time);
    println!("completed mutation in {}", format_duration(duration))
}

fn parse_files() -> (MutationConfig, Vec<Spell>) {
    let config: MutationConfig = serde_json::from_str(
        &*fs::read_to_string("../assets/config.json")
            .expect("failed to load config")
    ).expect("failed to parse config");

    let mut spells: Vec<Spell> = serde_json::from_str(
        &*fs::read_to_string(&config.input_file)
            .expect("failed to load spells")
    ).expect("failed to parse spells");

    spells.sort_unstable();

    (config, spells)
}
