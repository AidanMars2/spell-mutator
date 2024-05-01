use std::collections::{HashMap, HashSet};
use std::fs;

use serde::{Deserialize, Serialize};
use zspell::Dictionary;

#[derive(Serialize, Deserialize, Eq, PartialEq, Ord, PartialOrd)]
struct Spell {
    level: u8,
    school: String,
    name: String,
    source: String,
    cast_time: String,
    components: String,
    concentration: bool,
    ritual: bool,
    #[serde(default)]
    words: Vec<String>
}

impl Spell {
    fn write_mutations(
        &self,
        dictionary: &HashMap<String, HashSet<String>>
    ) -> (String, usize) {

        let spell_mutations = self.get_spell_mutations(dictionary);
        let spells_total = spell_mutations.len();
        let spells = spell_mutations.into_iter()
            .collect::<Vec<_>>()
            .into_iter()
            .map(|str| str + "\n")
            .collect::<String>();

        let beginning = self.write_spell_information();

        (beginning + &*spells, spells_total)
    }

    fn get_spell_mutations(
        &self,
        dictionary: &HashMap<String, HashSet<String>>
    ) -> Vec<String> {
        let mut spells: HashSet<String> = HashSet::new();

        let mut words_mut = self.words.clone();
        for index in 0..self.words.len() {
            let mutated_words = dictionary
                .get(&self.words[index].to_lowercase())
                .expect(format!("failed to get word {}", words_mut[index]).as_str())
                .clone();

            for word in mutated_words.into_iter() {
                words_mut[index] = word;
                spells.insert(
                    words_mut
                        .clone()
                        .into_iter()
                        .map(|s| s + " ")
                        .collect::<String>()
                );
            }
            words_mut[index] = self.words[index].clone()
        }

        spells.remove(&(self.name.to_lowercase() + " "));
        let mut spells_vec = spells
            .into_iter()
            .map(|it| it.trim().to_string())
            .collect::<Vec<_>>();
        spells_vec.sort_unstable();
        spells_vec
    }

    fn write_spell_level(&self) -> String {
        match self.level {
            0 => "Cantrip".to_string(),
            1 => "1st level".to_string(),
            2 => "2nd level".to_string(),
            3 => "3rd level".to_string(),
            n => n.to_string() + "th"
        }
    }

    fn write_spell_information(&self) -> String {
        let level_school = if self.level == 0 {
            format!("{} Cantrip", self.school)
        } else {
            format!("{} level {}", self.write_spell_level(), self.school)
        };

        format!(
            "\n{}, {}, {}, {}{} ({})\n",
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

fn main() {
    let spells_json = fs::read_to_string("spells.json")
        .expect("failed to load spells from file");
    let mut spells: Vec<Spell> = serde_json::from_str(&spells_json).expect("failed to parse spells");
    println!("initial spell count: {}", spells.len());
    spells.sort_unstable();

    #[cfg(feature = "frequency")]
    let mut word_frequency: HashMap<String, u32> = HashMap::new();

    let words: HashSet<String> = spells.iter_mut().map(|spell: &mut Spell| {
        let words: Vec<String> = spell.name
            .trim()
            .to_lowercase()
            .split(" ")
            .map(|s| { s.to_string() })
            .collect();
        spell.words.append(&mut (words.clone()));
        #[cfg(feature = "frequency")]
        {
            for word in words.iter() {
                word_frequency.insert(
                    word.clone(),
                    word_frequency.get(word).unwrap_or_else(|| &0) + 1,
                );
            }
        }
        words
    }).flatten().collect();

    #[cfg(feature = "frequency")]
    {
        println!("word frequency: ");

        let mut word_freq = word_frequency.into_iter().collect::<Vec<_>>();
        word_freq.sort_by_key(|tuple| tuple.1);

        for (word, freq) in word_freq {
            println!("{}: {}", word, freq)
        }
    }

    let dict = get_dict();

    println!("mutating words");
    println!("- initial word count: {}", words.len());

    let mut mutated_words: HashMap<String, HashSet<String>> = HashMap::new();
    for word in words.into_iter() {
        let mutated = mutate_word(&word, &dict);
        mutated_words.insert(word, mutated);
    }
    let total_words: usize = mutated_words.iter()
        .map(|entry| entry.1.len())
        .sum();

    println!("- mutated word count: {}", total_words);

    println!("formatting spells");
    let mut total_spells = 0usize;
    fs::write("spells_mutated.txt", spells.iter().map(|spell: &Spell| {
        let spells = spell.write_mutations(&mutated_words);
        total_spells += spells.1;

        spells.0
    }).collect::<String>()).expect("failed to write mutated spells to file");
    println!("- total mutated spell count: {}", total_spells)
}

fn mutate_word(word: &str, dictionary: &Dictionary) -> HashSet<String> {
    if word == "of" || word == "or" {
        // of has 20-ish different mutations, these are the interesting ones
        return HashSet::from([
            "on".to_string(),
            "or".to_string(),
            "oaf".to_string(),
            "if".to_string(),
            "of".to_string()
        ])
    }

    let chars = word.chars().collect::<Vec<char>>();
    let mut mutated_words = HashSet::new();

    let check_sentence = chars.iter().any(|char| !char.is_ascii_alphabetic());
    let check = |string: &str| {
        if check_sentence {
            dictionary.check(string)
        } else {
            dictionary.check_word(string)
        }
    };

    // change letter
    let mut mut_chars = chars.clone();
    for index in 0..chars.len() {
        for letter in 'a'..='z' {
            mut_chars[index] = letter;
            let word: String = mut_chars.iter().collect();
            if check(&word) {
                mutated_words.insert(word);
            }
        }

        mut_chars[index] = chars[index]
    }
    drop(mut_chars);

    // add letter
    let mut mut_chars = chars.clone();
    mut_chars.insert(0, 'a');
    for index in 0..=chars.len() {
        for letter in 'a'..='z' {
            mut_chars[index] = letter;
            let word: String = mut_chars.iter().collect();
            if check(&word) {
                mutated_words.insert(word);
            }
        }

        if index < chars.len() {
            mut_chars[index] = chars[index]
        }
    }
    drop(mut_chars);

    // remove letter
    let mut mut_chars = chars.clone();
    mut_chars.remove(0);
    for index in 0..chars.len() {
        let word: String = mut_chars.iter().collect();
        if check(&word) {
            mutated_words.insert(word);
        }

        if index < mut_chars.len() {
            mut_chars[index] = chars[index + 1]
        }
    }
    drop(mut_chars);

    // split word
    if word.len() > 7 {
        let mut_chars = chars.clone();
        for index in 0..chars.len() {
            if index < 4 || index > chars.len() - 4 {
                continue;
            }

            let word_one = mut_chars[..index].into_iter().collect::<String>();
            let word_two = mut_chars[index..].into_iter().collect::<String>();

            if !(dictionary.check_word(&word_one) &&
                dictionary.check_word(&word_two)) {
                continue;
            }

            let mutated_words_one = mutate_word(&word_one, dictionary);
            let mutated_words_two = mutate_word(&word_two, dictionary);

            for mut_word_one in mutated_words_one.into_iter() {
                let new_word = format!("{} {}", mut_word_one, word_two);
                // println!("new word: {}", new_word);
                mutated_words.insert(new_word);
            }

            for mut_word_two in mutated_words_two.into_iter() {
                let new_word = format!("{} {}", word_one, mut_word_two);
                // println!("new word: {}", new_word);
                mutated_words.insert(new_word);
            }
        }
    }

    mutated_words.remove(word);
    // remove generally uninteresting mutations
    mutated_words.remove(&(word.to_string() + "y"));
    mutated_words
}

fn get_dict() -> Dictionary {
    let aff_content = fs::read_to_string("lang_en_US.aff")
        .expect("failed to load lang config");
    let dict_content = fs::read_to_string("lang_en_US_DICT.dic")
        .expect("failed to load dictionary");

    zspell::builder()
        .dict_str(&dict_content)
        .config_str(&aff_content)
        .build()
        .expect("failed to init language")
}
