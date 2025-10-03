use std::mem;
use std::mem::ManuallyDrop;
use std::ops::{Deref, Index};
use std::ptr::NonNull;

pub(super) struct RemoveCharIter {
    chars: Box<[u8]>,
    partial_len: usize,
    idx: usize,
}

impl RemoveCharIter {
    pub(super) fn new(chars: Box<[u8]>) -> Self {
        let partial_len = chars.len() - 1;
        Self {
            partial_len,
            chars,
            idx: 0,
        }
    }

    pub(super) fn next(&mut self) -> Option<&[u8]> {
        while self.idx < self.partial_len {
            self.chars.swap(0, self.idx);
            let prev_char = if self.idx != 0 {
                self.chars.get(self.idx).copied().unwrap_or(b' ')
            } else {
                b' '
            };
            self.idx += 1;
            let next_char = self.chars.get(self.idx).copied().unwrap_or(b' ');
            // skip spaces and single letter words
            if !self.chars[0].is_ascii_alphabetic()
                || (!prev_char.is_ascii_alphabetic() && !next_char.is_ascii_alphabetic())
            {
                continue;
            }
            return Some(&self.chars[1..self.partial_len]);
        }
        None
    }

    pub(super) fn finish(mut self) -> Box<[u8]> {
        if self.partial_len > 0 && self.idx > 0 {
            let removed_char = self.chars[0];
            self.chars.copy_within(1..self.idx, 0);
            self.chars[self.idx - 1] = removed_char;
        }
        self.chars
    }
}
