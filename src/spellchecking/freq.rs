use crate::spellchecking::{CheckResult, SpellChecker};
use std::collections::HashMap;
use std::fs;
use std::iter::once;
use std::sync::LazyLock;
use itertools::Itertools;

peg::parser!(
    grammar freq_parser() for str {
        rule _() = " "*

        rule new_line() = "\r"? "\n"

        rule word() -> Option<&'input str> =
            r:$("("? !"-" ['a'..='z' | 'A'..='Z' | '-' | '\'' | '.' | '/']+ ")"?) "!"? "*"?
        {
            if r.contains(|c| !char::is_ascii_alphabetic(&c)) {
                return None
            }
            Some(r)
        }

        rule inflection_list() -> Vec<&'input str> = new_line() "    " infs:word() ** (_ "," _) {
            infs.into_iter().filter_map(|x| x).collect_vec()
        }

        rule head_word() -> (Option<&'input str>, Vec<&'input str>) =
            w:word() inf:inflection_list()? { (w, inf.unwrap_or_default()) }

        rule section() -> Vec<(Option<&'input str>, Vec<&'input str>)> =
            "-"+ _ ['0'..='9']+ _ "-"+ new_line()
            words:head_word() ** new_line() { words }

        pub rule dict() -> Vec<Vec<(Option<&'input str>, Vec<&'input str>)>> = s:section() ** new_line() {s}
    }
);

static FRQ_STRING: LazyLock<String> = LazyLock::new(|| {
    fs::read_to_string("./dicts/12dicts-6.0.2/Lemmatized/2+2+3frq.txt")
        .expect("failed to read dictionary")
});

pub struct FreqSpellChecker {
    words: HashMap<&'static str, usize, rapidhash::fast::RandomState>,
    relations: Vec<(u8, Vec<usize>)>,
}

impl FreqSpellChecker {
    pub fn new() -> Self {
        let parsed = freq_parser::dict(&FRQ_STRING)
            .expect("couldn't parse dictionary");

        let mut relations = vec![];
        let mut words = HashMap::with_hasher(rapidhash::fast::RandomState::new());

        for (freq_idx, section) in parsed.into_iter().enumerate() {
            let freq_idx = freq_idx as u8;
            for (head, mut inflections) in section {
                if let Some(head) = head {
                    inflections.push(head)
                }
                let mut mesh = vec![];
                for word in inflections {
                    let idx = *words.entry(word).or_insert_with(|| {
                        let next_idx = relations.len();
                        relations.push((freq_idx, vec![]));
                        next_idx
                    });
                    mesh.push(idx)
                }
                let mut extension = vec![];
                for index in &mesh {
                    let (section, relations) = &mut relations[*index];
                    if *section != freq_idx {
                        *section = freq_idx;
                        extension.extend(relations.iter().copied())
                    }
                }
                mesh.extend(extension);
                for index in &mesh {
                    let relations = &mut relations[*index].1;
                    relations.clear();
                    relations.extend(&mesh);
                }
            }
        }

        Self { words, relations }
    }
}

impl SpellChecker for FreqSpellChecker {
    fn name(&self) -> &'static str {
        "Frequency"
    }

    fn check(&self, original: &str, word: &str) -> CheckResult {
        char::is_ascii_alphabetic(&'b');
        if original == word {
            return CheckResult::SUCCESS;
        }
        if let Some(word_idx) = self.words.get(word) {
            let word_freq_code = self.relations[*word_idx].0.saturating_sub(16);
            if let Some(original_idx) = self.words.get(original) {
                let relations = &self.relations[*word_idx].1;
                if relations.contains(original_idx) {
                    return CheckResult::new(word_freq_code + 100);
                }
            }
            return CheckResult::new(word_freq_code);
        }
        CheckResult::FAIL
    }
}
