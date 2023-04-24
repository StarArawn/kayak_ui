use super::MaterialUI;
use std::hash::Hash;

pub struct MaterialUIKey<M: MaterialUI> {
    // pub mesh_key: Mesh2dPipelineKey,
    pub bind_group_data: M::Data,
}

impl<M: MaterialUI> Eq for MaterialUIKey<M> where M::Data: PartialEq {}

impl<M: MaterialUI> PartialEq for MaterialUIKey<M>
where
    M::Data: PartialEq,
{
    fn eq(&self, other: &Self) -> bool {
        //self.mesh_key == other.mesh_key &&
        self.bind_group_data == other.bind_group_data
    }
}

impl<M: MaterialUI> Clone for MaterialUIKey<M>
where
    M::Data: Clone,
{
    fn clone(&self) -> Self {
        Self {
            // mesh_key: self.mesh_key,
            bind_group_data: self.bind_group_data.clone(),
        }
    }
}

impl<M: MaterialUI> Hash for MaterialUIKey<M>
where
    M::Data: Hash,
{
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        // self.mesh_key.hash(state);
        self.bind_group_data.hash(state);
    }
}
