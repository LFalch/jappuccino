use std::{collections::BTreeMap, ops::Index};

use crate::descriptor::AnyDescriptor;

#[derive(Debug, Default, Clone)]
pub struct MemberTable(
    BTreeMap<Box<str>, BTreeMap<AnyDescriptor, u16>>
);

impl MemberTable {
    pub const fn new() -> Self {
        Self(BTreeMap::new())
    }
    pub fn insert(&mut self, name: impl Into<Box<str>>, descriptor: impl Into<AnyDescriptor>, offset: u16) {
        self.0.entry(name.into()).or_default().insert(descriptor.into(), offset);
    }
    pub fn get(&self, name: &str, descriptor: &AnyDescriptor) -> Option<u16> {
        self.0.get(name)?.get(descriptor).copied()
    }
}

impl Index<(&str, &AnyDescriptor)> for MemberTable {
    type Output = u16;
    #[track_caller]
    fn index(&self, (name, descriptor): (&str, &AnyDescriptor)) -> &Self::Output {
        &self.0[name][descriptor]
    }
}
