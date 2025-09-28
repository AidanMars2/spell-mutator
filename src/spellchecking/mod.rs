use crate::spellchecking::CheckResult::Fail;

pub mod old;
pub mod lemma;

pub trait SpellChecker {
    fn check(&self, original: &str, word: &str) -> CheckResult;

    fn check_split(&self, original: &str, string: &str) -> CheckResult {
        string.split(' ')
            .map(|it| self.check(original, it))
            .reduce(CheckResult::min)
            .unwrap_or(CheckResult::Succes)
    }
}

#[derive(Copy, Clone)]
pub enum CheckResult {
    Succes,
    Maybe,
    Fail
}

impl CheckResult {
    pub fn min(self, rhs: CheckResult) -> CheckResult {
        match (self, rhs) {
            (Self::Succes, it) => it,
            (Self::Maybe, Self::Succes) => Self::Maybe,
            (Self::Maybe, it) => it,
            (Fail, _) => Fail
        }
    }
}