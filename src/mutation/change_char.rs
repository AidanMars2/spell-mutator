use std::mem;
use std::mem::MaybeUninit;

pub(super) struct ChangeCharIter {
    chars: Box<[u8]>,
    partial_len: usize,
    idx: usize,
    original_letter: u8
}

impl ChangeCharIter {
    pub(super) fn new(mut chars: Box<[u8]>) -> Self {
        let partial_len = chars.len() - 1;
        Self {
            original_letter: mem::replace(&mut chars[0], b'a' - 1), idx: 0,
            chars, partial_len,
        }
    }

    pub(super) fn next(&mut self) -> Option<&[u8]> {
        if self.idx >= self.partial_len {
            return None
        }
        let letter = &mut self.chars[self.idx];
        if *letter < b'z' {
            *letter += 1;
            return Some(&self.chars);
        }

        loop {
            self.chars[self.idx] = self.original_letter;
            self.idx += 1;
            return if self.idx < self.partial_len {
                let next = &mut self.chars[self.idx];
                self.original_letter = *next;
                if !next.is_ascii_alphabetic() {
                    continue
                }
                *next = b'a';
                Some(&self.chars)
            } else {
                None
            }
        }
    }

    pub(super) fn get(&self) -> &[u8] {
        &self.chars[..self.partial_len]
    }
    
    pub(super) fn finish(mut self) -> Box<[u8]> {
        if self.idx < self.chars.len() {
            self.chars[self.idx] = self.original_letter;
        }
        self.chars
    }
}