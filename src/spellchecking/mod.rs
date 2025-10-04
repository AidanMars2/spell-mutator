use itertools::{EitherOrBoth, Itertools};
use serde::{Deserialize, Serialize};
use std::cmp::{max, min};
use std::fmt::Display;

pub mod freq;
pub mod lemma;
pub mod old;

pub trait SpellChecker: Send + Sync {
    fn name(&self) -> &'static str;

    fn check(&self, original: &str, word: &str) -> CheckResult;

    fn check_split(&self, original: &str, string: &str) -> CheckResult {
        let mut result = CheckResult::SUCCESS;
        for word in string.split(' ') {
            result = result.worst(self.check(original, word));
        }
        result
    }
}

#[derive(Copy, Clone, Hash, Ord, PartialOrd, Eq, PartialEq, Debug, Serialize, Deserialize)]
#[serde(transparent)]
pub struct CheckResult {
    value: u8,
}

impl CheckResult {
    pub const FAIL: Self = Self { value: u8::MAX };
    pub const SUCCESS: Self = Self { value: 0 };

    pub fn new(code: u8) -> Self {
        Self { value: code }
    }

    #[inline]
    pub fn is_fail(self) -> bool {
        self.value == u8::MAX
    }

    pub fn value(self) -> u8 {
        self.value
    }

    pub fn worst(self, rhs: Self) -> Self {
        Self {
            value: max(self.value(), rhs.value),
        }
    }
}

impl Display for CheckResult {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let str = if self.value == 0 {
            String::new()
        } else {
            format!("#({}) ", self.value)
        };
        write!(f, "{}", str)
    }
}
