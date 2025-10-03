use std::cell::RefMut;
use std::mem;
use std::ptr::slice_from_raw_parts;

pub(super) struct AddCharIter {
    chars: Box<[u8]>,
    idx: usize,
}

impl AddCharIter {
    pub(super) fn new(mut chars: Box<[u8]>) -> Self {
        chars.copy_within(..chars.len() - 1, 1);
        chars[0] = b'a' - 1;
        Self { chars, idx: 0 }
    }

    pub(super) fn next(&mut self) -> Option<&[u8]> {
        if self.idx >= self.chars.len() {
            return None;
        }
        let prev_letter = self
            .idx
            .checked_sub(1)
            .map(|prev_idx| self.chars[prev_idx])
            .unwrap_or(b' ');
        let next_letter = self.chars.get(self.idx + 1).copied().unwrap_or(b' ');

        let letter = &mut self.chars[self.idx];
        if *letter < b'z' && *letter >= b'a' - 1 {
            *letter += 1;
            return Some(&self.chars);
        }
        if *letter == b'z' && prev_letter.is_ascii_alphabetic() && next_letter.is_ascii_alphabetic()
        {
            *letter = b' ';
            return Some(&self.chars);
        }

        self.idx += 1;
        if self.idx < self.chars.len() {
            self.chars.swap(self.idx - 1, self.idx);
            Some(&self.chars)
        } else {
            None
        }
    }

    pub(super) fn get(&self) -> &[u8] {
        &self.chars
    }

    pub(super) fn finish(mut self) -> Box<[u8]> {
        if self.idx < self.chars.len() {
            self.chars.copy_within(self.idx + 1.., self.idx);
        }
        self.chars
    }
}
