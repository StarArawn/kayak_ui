use crate::render::unified::pipeline::UnifiedPipelineKey;

use super::MaterialUI;
use std::hash::Hash;

pub struct MaterialUIKey<M: MaterialUI> {
    pub unified_key: UnifiedPipelineKey,
    pub bind_group_data: M::Data,
}

impl<M: MaterialUI> Eq for MaterialUIKey<M> where M::Data: PartialEq {}

impl<M: MaterialUI> PartialEq for MaterialUIKey<M>
where
    M::Data: PartialEq,
{
    fn eq(&self, other: &Self) -> bool {
        self.unified_key == other.unified_key && self.bind_group_data == other.bind_group_data
    }
}

impl<M: MaterialUI> Clone for MaterialUIKey<M>
where
    M::Data: Clone,
{
    fn clone(&self) -> Self {
        Self {
            unified_key: self.unified_key,
            bind_group_data: self.bind_group_data.clone(),
        }
    }
}

impl<M: MaterialUI> Hash for MaterialUIKey<M>
where
    M::Data: Hash,
{
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.unified_key.hash(state);
        self.bind_group_data.hash(state);
    }
}
