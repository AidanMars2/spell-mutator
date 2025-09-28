use itertools::{EitherOrBoth, Itertools};
use crate::spellchecking::CheckResult::Fail;

pub mod old;
pub mod lemma;

pub trait SpellChecker {
    fn check(&self, original: &str, word: &str) -> CheckResult;

    fn check_split(&self, original: &str, string: &str) -> CheckResult {
        original.split(' ')
            .zip_longest(string.split(' '))
            .map(|tuple| {
                match tuple {
                    EitherOrBoth::Both(orig, string) => self.check(orig, string),
                    EitherOrBoth::Right(string) => self.check("", string),
                    _ => unreachable!("more words in original than mutation")
                }
            })
            .reduce(CheckResult::min)
            .unwrap_or(CheckResult::Success)
    }
}

#[derive(Copy, Clone)]
pub enum CheckResult {
    Success,
    Maybe,
    Fail
}

impl CheckResult {
    pub fn min(self, rhs: CheckResult) -> CheckResult {
        match (self, rhs) {
            (Self::Success, it) => it,
            (Self::Maybe, Self::Success) => Self::Maybe,
            (Self::Maybe, it) => it,
            (Fail, _) => Fail
        }
    }
}