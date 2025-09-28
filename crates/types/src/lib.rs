use std::cmp::Ordering;
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};

#[derive(Serialize, Deserialize, Eq, PartialEq, Ord, PartialOrd, Debug)]
pub struct Spell {
    pub name: String,
    pub level: u8,
    pub school: String,
    pub source: String,
    #[serde(default)]
    pub cast_time: String,
    #[serde(default)]
    pub components: String,
    #[serde(default)]
    pub concentration: bool,
    #[serde(default)]
    pub ritual: bool,
    #[serde(default)]
    pub mutations: MutationResult
}

impl Spell {

    fn write_spell_level(&self) -> String {
        match self.level {
            0 => "Cantrip".to_string(),
            1 => "1st".to_string(),
            2 => "2nd".to_string(),
            3 => "3rd".to_string(),
            n => n.to_string() + "th"
        }
    }

    pub fn write_spell_information(&self) -> String {
        let level_school = if self.level == 0 {
            format!("{} Cantrip", self.school)
        } else {
            format!("{} level {}", self.write_spell_level(), self.school)
        };

        format!(
            "\n{},\n{},\n{}, {}{} ({})\n\n",
            self.name,
            level_school,
            self.cast_time,
            self.components,
            match () {
                () if self.concentration && self.ritual => "<CR>",
                () if self.concentration => "<C>",
                () if self.ritual => "<R>",
                () => ""
            },
            self.source
        )
    }
}

#[derive(Serialize, Deserialize)]
pub struct Overrides {
    #[serde(default)]
    pub allow_split: HashMap<String, usize>,
}

#[derive(Serialize, Deserialize)]
pub struct MutationConfig {
    pub input_file: String,
    pub output_file: String,
    pub formatted_file: String,
    pub overrides_file: String,
    pub diagnostics_file: String,
    pub mutated_words_file: String,
    pub mutation_depth: usize,
    pub advanced_diagnostics: bool,
    pub omit_zero_mutation_spells: bool
}


#[derive(Default, Clone, Serialize, Deserialize, Debug, Eq, PartialEq)]
pub struct MutationResult {
    pub mutations: HashSet<String>,
    pub maybe_mutations: HashSet<String>,
}

impl Ord for MutationResult {
    fn cmp(&self, _other: &Self) -> Ordering {
        Ordering::Equal
    }
}

impl PartialOrd for MutationResult {
    fn partial_cmp(&self, _other: &Self) -> Option<Ordering> {
        Some(Ordering::Equal)
    }
}

impl MutationResult {
    pub fn new() -> Self {
        Self {
            mutations: HashSet::new(),
            maybe_mutations: HashSet::new()
        }
    }
}
