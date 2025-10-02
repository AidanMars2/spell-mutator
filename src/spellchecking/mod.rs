use itertools::{EitherOrBoth, Itertools};
use crate::spellchecking::CheckResult::Fail;

pub mod old;
pub mod lemma;

pub trait SpellChecker {
    fn name(&self) -> &'static str;

    fn check(&self, original: &str, word: &str) -> CheckResult;

    fn check_split(&self, original: &str, string: &str) -> CheckResult {
        let mut result = CheckResult::Success;
        for word in string.split(' ') {
            result = result.min(self.check(original, word));
        }
        result
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