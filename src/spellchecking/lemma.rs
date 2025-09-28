use std::collections::HashMap;
use std::fs;
use std::iter::once;
use std::sync::LazyLock;
use crate::spellchecking::CheckResult::{Fail, Maybe, Succes};
use crate::spellchecking::{CheckResult, SpellChecker};

static LEMMA_STRING: LazyLock<String> = LazyLock::new(|| fs::read_to_string
    ("./dicts/12dicts-6.0.2/Lemmatized/2+2+3lem.txt")
    .expect("failed to read dictionary"));

type ParsedWord<'a> = (&'a str, Vec<&'a str>);

peg::parser! {
    grammar lemma_parser() for str {
        rule _() = " "*

        rule word() -> &'input str = $(['a'..='z' | 'A'..='Z' | '-']+)

        rule cross_ref() -> Vec<&'input str> = _ "->" _ "[" _ refs:(word() ** (_ "," _)) _ "]" { refs }

        rule inflection() -> ParsedWord<'input> =
            w:word() refs:cross_ref()? { (w, refs.unwrap_or(Vec::new())) }

        rule inflection_list() -> Vec<ParsedWord<'input>> = "\n    " infs:inflection() ** (_ "," _) { infs }

        rule head_word() -> (ParsedWord<'input>, Vec<ParsedWord<'input>>) =
            w:inflection() inf:inflection_list()?{ (w, inf.unwrap_or(Vec::new())) }

        pub rule dict() -> Vec<(ParsedWord<'input>, Vec<ParsedWord<'input>>)> = head_word() ** "\n"
    }
}

pub struct LemmaSpellChecker {
    words: HashMap<&'static str, usize>,
    relations: Vec<Vec<usize>>,
}

impl LemmaSpellChecker {
    pub fn new() -> Self {
        let parsed = lemma_parser::dict(LEMMA_STRING.as_str())
            .expect("couldn't parse dictionary");

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
    fn check(&self, original: &str, word: &str) -> CheckResult {
        if let Some(word_idx) = self.words.get(word) {
            if let Some(original_idx) = self.words.get(original) {
                let relations = &self.relations[*word_idx];
                if relations.contains(original_idx) {
                    return Maybe
                }
            }
            return Succes
        }
        Fail
    }
}