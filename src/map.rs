use std::io;
use std::marker::PhantomData;
use std::ops::{Deref, DerefMut};

use bytehash::ByteHash;
use owning_ref::{OwningRef, OwningRefMut};

use crate::branch::{Branch, BranchMut};
use crate::compound::Compound;
use crate::iter::{LeafIter, LeafIterMut};
use crate::search::{First, Method};

pub trait KVPair<K, V>: Into<(K, V)> + From<(K, V)> {
    fn key(&self) -> &K;
    fn val(&self) -> &V;
    fn val_mut(&mut self) -> &mut V;
    fn into_val(self) -> V;
}

impl<K, V> KVPair<K, V> for (K, V) {
    fn key(&self) -> &K {
        &self.0
    }

    fn val(&self) -> &V {
        &self.1
    }

    fn val_mut(&mut self) -> &mut V {
        &mut self.1
    }

    fn into_val(self) -> V {
        self.1
    }
}

/// A path to a leaf in a map Compound
pub struct ValPath<'a, K, V, C, H>
where
    C: Compound<H>,
    C::Leaf: KVPair<K, V>,
    H: ByteHash,
{
    branch: Branch<'a, C, H>,
    _marker: PhantomData<(K, V)>,
}

/// A path to a mutable leaf in a map Compound
pub struct ValPathMut<'a, K, V, C, H>
where
    C: Compound<H>,
    C::Leaf: KVPair<K, V>,
    H: ByteHash,
{
    branch: BranchMut<'a, C, H>,
    _marker: PhantomData<(K, V)>,
}

impl<'a, K, V, C, H> ValPath<'a, K, V, C, H>
where
    C: Compound<H>,
    C::Leaf: KVPair<K, V>,
    H: ByteHash,
    K: PartialEq + Eq,
{
    /// Creates a new `ValPath`, when leaf is found and key matches
    pub fn new<'m, M>(
        node: &'a C,
        method: &mut M,
        key: &K,
    ) -> io::Result<Option<Self>>
    where
        M: Method,
    {
        Ok(Branch::new(node, method)?.filter(|b| b.key() == key).map(
            |branch| ValPath {
                branch,
                _marker: PhantomData,
            },
        ))
    }
}

impl<'a, K, V, C, H> ValPathMut<'a, K, V, C, H>
where
    C: Compound<H>,
    C::Leaf: KVPair<K, V>,
    H: ByteHash,
    K: PartialEq + Eq,
{
    /// Creates a new `ValPathMut`, when leaf is found and key matches
    pub fn new<'m, M>(
        node: &'a mut C,
        method: &mut M,
        key: &K,
    ) -> io::Result<Option<Self>>
    where
        M: Method,
    {
        Ok(BranchMut::new(node, method)?
            .filter(|b| b.key() == key)
            .map(|branch| ValPathMut {
                branch,
                _marker: PhantomData,
            }))
    }
}

impl<'a, K, V, C, H> Deref for ValPath<'a, K, V, C, H>
where
    C: Compound<H>,
    C::Leaf: KVPair<K, V>,
    H: ByteHash,
{
    type Target = V;

    fn deref(&self) -> &Self::Target {
        self.branch.val()
    }
}

impl<'a, K, V, C, H> Deref for ValPathMut<'a, K, V, C, H>
where
    C: Compound<H>,
    C::Leaf: KVPair<K, V>,
    H: ByteHash,
{
    type Target = V;

    fn deref(&self) -> &Self::Target {
        self.branch.val()
    }
}

impl<'a, K, V, C, H> DerefMut for ValPathMut<'a, K, V, C, H>
where
    C: Compound<H>,
    C::Leaf: KVPair<K, V>,
    H: ByteHash,
{
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.branch.val_mut()
    }
}

pub struct ValIter<'a, C, K, V, M, H>(
    LeafIter<'a, C, M, H>,
    PhantomData<(K, V)>,
)
where
    C: Compound<H>,
    H: ByteHash;

pub struct ValIterMut<'a, C, K, V, M, H>(
    LeafIterMut<'a, C, M, H>,
    PhantomData<(K, V)>,
)
where
    C: Compound<H>,
    H: ByteHash;

pub struct KeyIter<'a, C, K, V, M, H>(
    LeafIter<'a, C, M, H>,
    PhantomData<(K, V)>,
)
where
    C: Compound<H>,
    H: ByteHash;

/// Compound can be iterated over like a map
pub trait KeyValIterable<K, V, H>
where
    Self: Compound<H>,
    Self::Leaf: KVPair<K, V>,
    H: ByteHash,
{
    /// Iterator over the values of the map
    fn values(&self) -> ValIter<Self, K, V, First, H>;

    /// Iterator over the mutable values of the map
    fn values_mut(&mut self) -> ValIterMut<Self, K, V, First, H>;

    /// Iterator over the keys of the map
    fn keys(&mut self) -> KeyIter<Self, K, V, First, H>;
}

impl<C, K, V, H> KeyValIterable<K, V, H> for C
where
    C: Compound<H>,
    C::Leaf: KVPair<K, V>,
    H: ByteHash,
{
    fn values(&self) -> ValIter<Self, K, V, First, H> {
        ValIter(LeafIter::Initial(self, First), PhantomData)
    }

    fn values_mut(&mut self) -> ValIterMut<Self, K, V, First, H> {
        ValIterMut(LeafIterMut::Initial(self, First), PhantomData)
    }

    fn keys(&mut self) -> KeyIter<Self, K, V, First, H> {
        KeyIter(LeafIter::Initial(self, First), PhantomData)
    }
}

impl<'a, C, K, V, M, H> Iterator for ValIter<'a, C, K, V, M, H>
where
    C: Compound<H>,
    C::Leaf: KVPair<K, V>,
    M: 'a + Method,
    K: 'a,
    V: 'a,
    H: ByteHash,
{
    type Item = io::Result<&'a V>;

    fn next(&mut self) -> Option<Self::Item> {
        self.0.next().map(|result| result.map(KVPair::val))
    }
}

impl<'a, C, K, V, M, H> Iterator for ValIterMut<'a, C, K, V, M, H>
where
    C: Compound<H>,
    C::Leaf: KVPair<K, V>,
    M: 'a + Method,
    K: 'a,
    V: 'a,
    H: ByteHash,
{
    type Item = io::Result<&'a mut V>;

    fn next(&mut self) -> Option<Self::Item> {
        self.0.next().map(|r| r.map(KVPair::val_mut))
    }
}

impl<'a, C, K, V, M, H> Iterator for KeyIter<'a, C, K, V, M, H>
where
    C: Compound<H>,
    C::Leaf: KVPair<K, V>,
    M: 'a + Method,
    K: 'a,
    V: 'a,
    H: ByteHash,
{
    type Item = io::Result<&'a K>;

    fn next(&mut self) -> Option<Self::Item> {
        self.0.next().map(|result| result.map(KVPair::key))
    }
}

/// Value reference trait to hide generic arguments to users of the library
pub trait ValRef<'a, V>: Deref<Target = V> + 'a
where
    Self: Sized,
{
    /// Wrap the ValPath in an OwningRef
    fn wrap<V2, F>(self, f: F) -> OwningRef<Box<Self>, V2>
    where
        V2: 'a,
        F: FnOnce(&Self) -> &V2,
    {
        OwningRef::new(Box::new(self)).map(f)
    }
}
impl<'a, T, V> ValRef<'a, V> for T where T: Deref<Target = V> + 'a {}

/// Mutable value reference trait to hide generic arguments to users of the library
pub trait ValRefMut<'a, V>: DerefMut<Target = V> + 'a
where
    Self: Sized,
{
    /// Wrap the ValPathMut in an OwningRef
    fn wrap_mut<V2, F>(self, f: F) -> OwningRefMut<Box<Self>, V2>
    where
        V2: 'a,
        F: FnOnce(&mut Self) -> &mut V2,
    {
        OwningRefMut::new(Box::new(self)).map_mut(f)
    }
}

impl<'a, T, V> ValRefMut<'a, V> for T where
    T: Deref<Target = V> + 'a + DerefMut + 'a
{
}
