use std::collections::{HashMap, HashSet};
use serde::{Deserialize, Serialize};

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
    pub mutations: Vec<String>
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
    pub overrides: HashMap<String, Vec<String>>,
    #[serde(default)]
    pub allow_split: HashMap<String, usize>,
    #[serde(default)]
    pub illegal_words: HashSet<String>,
    #[serde(default)]
    pub force_allow: HashSet<String>
}

#[derive(Serialize, Deserialize)]
pub struct MutationConfig {
    pub input_file: String,
    pub output_file: String,
    pub formatted_file: String,
    pub overrides_file: String,
    pub diagnostics_file: String,
    pub dic_file: String,
    pub aff_file: String,
    pub mutated_words_file: String,
    pub mutation_depth: u8,
    pub advanced_diagnostics: bool,
    pub omit_zero_mutation_spells: bool
}
