use bevy::{prelude::Handle, render::texture::Image, utils::HashMap};

#[derive(Debug, Clone)]
pub struct ImageManager {
    count: u16,
    mapping: HashMap<u16, Handle<Image>>,
    reverse_mapping: HashMap<Handle<Image>, u16>,
}

impl ImageManager {
    pub fn new() -> Self {
        Self {
            count: 0,
            mapping: HashMap::default(),
            reverse_mapping: HashMap::default(),
        }
    }

    pub fn get(&mut self, image_handle: &Handle<Image>) -> u16 {
        if let Some(id) = self.reverse_mapping.get(image_handle) {
            return *id;
        } else {
            let id = self.count;
            self.count += 1;
            self.mapping.insert(id, image_handle.clone());
            self.reverse_mapping.insert(image_handle.clone_weak(), id);
            return id;
        }
    }

    pub fn get_handle(&self, id: &u16) -> Option<&Handle<Image>> {
        self.mapping.get(id)
    }
}
