use crate::content::Content;
use crate::handle::{Handle, HandleRef};
use crate::ByteHash;

pub trait Method {
    fn select<C, H>(&mut self, handles: &[Handle<C, H>]) -> Option<usize>
    where
        C: Content<H>,
        H: ByteHash;
}

pub struct First;

impl Method for First {
    fn select<C, H>(&mut self, handles: &[Handle<C, H>]) -> Option<usize>
    where
        C: Content<H>,
        H: ByteHash,
    {
        for (i, h) in handles.iter().enumerate() {
            match h.inner() {
                HandleRef::Leaf(_) | HandleRef::Node(_) => return Some(i),
                HandleRef::None => (),
            }
        }
        None
    }
}
