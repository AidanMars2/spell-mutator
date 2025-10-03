use serde::{Deserialize, Serialize};
use std::collections::HashMap;

pub const MUTATED_SPELLS_JSON: &str = "spells_mutated.json";
pub const MUTATED_SPELLS_FILE: &str = "mutated spells.txt";
pub const MUTATED_WORDS_FILE: &str = "mutated words.txt";
pub const DIAGNOSTICS_FILE: &str = "diagnostics.txt";

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
}

impl Spell {
    fn write_spell_level(&self) -> String {
        match self.level {
            0 => "Cantrip".to_string(),
            1 => "1st".to_string(),
            2 => "2nd".to_string(),
            3 => "3rd".to_string(),
            n => n.to_string() + "th",
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
                () => "",
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
    pub overrides_file: String,
    pub output_dir: String,
    pub mutation_depth: usize,
    pub advanced_diagnostics: bool,
    pub omit_zero_mutation_spells: bool,
}
