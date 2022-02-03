use std::fmt::{Debug, Formatter};
use std::sync::Arc;
use crate::{Index, KayakContextRef};

type ChildrenBuilder = Arc<dyn Fn(Option<Index>, &mut KayakContextRef) + Send + Sync>;

/// A container for a function that generates child widgets
#[derive(Clone)]
pub struct Children(ChildrenBuilder);

impl Children {
    pub fn new(builder: ChildrenBuilder) -> Self {
        Self(builder)
    }
    pub fn build(&self, id: Option<Index>, context: &mut KayakContextRef) {
        self.0(id, context);
    }
}

impl Debug for Children {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Children").finish()
    }
}

impl PartialEq for Children {
    fn eq(&self, _: &Self) -> bool {
        // Never prevent "==" for being true because of this struct
        true
    }
}