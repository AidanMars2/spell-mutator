use std::cell::RefCell;
use std::mem;
use std::mem::MaybeUninit;
use std::rc::Rc;
use itertools::Itertools;
use crate::mutation::add_char::AddCharIter;
use crate::mutation::change_char::ChangeCharIter;
use crate::mutation::remove_char::RemoveCharIter;

pub struct MutateStringIter {
    add_char: Option<AddCharIter>,
    change_char: Option<ChangeCharIter>,
    remove_char: Option<RemoveCharIter>
}

impl MutateStringIter {
    pub fn new(chars: &[u8]) -> Self {
        Self {
            add_char: Some(AddCharIter::new(inc_size(chars))),
            change_char: None, remove_char: None
        }
    }
    
    pub fn next(&mut self) -> Option<&[u8]> {
        self.add_char = self.add_char.take()
            .and_then(|mut it| if it.next().is_some() { Some(it) } else {
                self.change_char = Some(ChangeCharIter::new(it.finish()));
                None
            });
        if let Some(iter) = &self.add_char {
            return Some(iter.get())
        }

        self.change_char = self.change_char.take()
            .and_then(|mut it| if it.next().is_some() { Some(it) } else {
                self.remove_char = Some(RemoveCharIter::new(it.finish()));
                None
            });
        if let Some(iter) = &self.change_char {
            return Some(iter.get())
        }

        if let Some(iter) = &mut self.remove_char {
            return iter.next()
        }
        None
    }
}

fn inc_size(chars: &[u8]) -> Box<[u8]> {
    let mut new_chars = Box::new_uninit_slice(chars.len() + 1);
    new_chars[chars.len()] = MaybeUninit::new(b'a');
    unsafe {
        new_chars[..chars.len()].copy_from_slice(mem::transmute(chars));
        new_chars.assume_init()
    }
}