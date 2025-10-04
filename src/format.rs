use crate::spellchecking::CheckResult;
use dashmap::DashMap;
use itertools::Itertools;
use std::collections::{HashMap, HashSet};
use std::fs;
use std::fs::File;
use std::io::Write;
use std::path::PathBuf;
use std::str::FromStr;
use types::{MutationConfig, Spell, MUTATED_SPELLS_FILE, MUTATED_SPELLS_JSON};

pub fn format_mutations(
    spells: Vec<Spell>,
    mutations: DashMap<&'static str, HashMap<String, HashMap<String, (CheckResult, usize)>>>,
    config: MutationConfig,
) {
    let mut output = PathBuf::from_str(&config.output_dir).unwrap();
    for (checker_name, mut mutations) in mutations {
        output.push(checker_name);
        fs::create_dir(&output);
        let mut output_files = vec![];
        for depth in 1..=config.mutation_depth {
            output.push(format!("{depth}deep"));
            output.push("mutated spells.txt");
            output_files.push(File::create(&output).expect("Failed to open output file"));
            output.pop();
            output.pop();
        }
        output.pop();
        for spell in &spells {
            if let Some(mutations) = mutations.remove(&spell.name) {
                let mut mutations = mutations
                    .into_iter()
                    .fold(HashMap::new(), |mut acc, (mutation, (check, depth))| {
                        acc.entry(check)
                            .or_insert_with(HashMap::new)
                            .entry(depth)
                            .or_insert_with(Vec::new)
                            .push(mutation);
                        acc
                    })
                    .into_iter()
                    .map(|(depth, checked)| (depth, checked.into_iter().collect_vec()))
                    .collect_vec();
                mutations.sort_unstable_by_key(|(x, _)| *x);
                let mut empties = vec![true; config.mutation_depth];

                for (check, mutations) in &mut mutations {
                    mutations.sort_unstable_by_key(|(depth, _)| *depth);
                    for (depth, mutations) in mutations {
                        mutations.sort_unstable();
                        for depth_idx in 0..*depth {
                            let mut target = &mut output_files[depth_idx];
                            if empties[depth_idx] {
                                target
                                    .write_all(spell.write_spell_information().as_bytes())
                                    .unwrap();
                                empties[depth_idx] = false;
                            }
                            for mutation in mutations.iter() {
                                write!(target, "{}{}\n", check, mutation).unwrap();
                            }
                        }
                    }
                }
            }
        }
    }
}
