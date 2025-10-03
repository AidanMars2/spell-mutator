use crate::spellchecking::{CheckResult, SpellChecker};
use std::collections::HashMap;
use std::fs;
use std::iter::once;
use std::sync::LazyLock;

static LEMMA_STRING: LazyLock<String> = LazyLock::new(|| {
    fs::read_to_string("./dicts/12dicts-6.0.2/Lemmatized/2+2+3lem.txt")
        .expect("failed to read dictionary")
});

type ParsedWord<'a> = (&'a str, Vec<&'a str>);

peg::parser! {
    grammar lemma_parser() for str {
        rule _() = " "*

        rule new_line() = "\r\n" / "\n"

        rule word() -> &'input str = r:$(['a'..='z' | 'A'..='Z' | '-']+) "!"? {r}

        rule cross_ref() -> Vec<&'input str> = _ "->" _ "[" _ refs:(word() ** (_ "," _)) _ "]" { refs }

        rule inflection() -> ParsedWord<'input> =
            w:word() refs:cross_ref()? { (w, refs.unwrap_or(Vec::new())) }

        rule inflection_list() -> Vec<ParsedWord<'input>> = new_line() "    " infs:inflection() ** (_ "," _) { infs }

        rule head_word() -> (ParsedWord<'input>, Vec<ParsedWord<'input>>) =
            w:inflection() inf:inflection_list()? { (w, inf.unwrap_or(Vec::new())) }

        pub rule dict() -> Vec<(ParsedWord<'input>, Vec<ParsedWord<'input>>)> = head_word() ** new_line()
    }
}

pub struct LemmaSpellChecker {
    words: HashMap<&'static str, usize>,
    relations: Vec<Vec<usize>>,
}

impl LemmaSpellChecker {
    pub fn new() -> Self {
        let parsed = lemma_parser::dict(LEMMA_STRING.as_str()).expect("couldn't parse dictionary");

        let mut relations = vec![];
        let mut words = HashMap::new();

        for (head, inflections) in parsed {
            let mut mesh = vec![];
            for parsed in inflections.iter().chain(once(&head)) {
                let idx = *words.entry(parsed.0).or_insert_with(|| {
                    let next_idx = relations.len();
                    relations.push(vec![]);
                    next_idx
                });
                mesh.push(idx)
            }
            for index in &mesh {
                relations[*index].extend(&mesh)
            }
        }

        Self { relations, words }
    }
}

impl SpellChecker for LemmaSpellChecker {
    fn name(&self) -> &'static str {
        "Lemma"
    }

    fn check(&self, original: &str, word: &str) -> CheckResult {
        if original == word {
            return CheckResult::SUCCESS;
        }
        if let Some(word_idx) = self.words.get(word) {
            if let Some(original_idx) = self.words.get(original) {
                let relations = &self.relations[*word_idx];
                if relations.contains(original_idx) {
                    return CheckResult::new(1);
                }
            }
            return CheckResult::SUCCESS;
        }
        CheckResult::FAIL
    }
}
