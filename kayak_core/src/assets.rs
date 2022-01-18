use std::{collections::HashMap, path::PathBuf};

use crate::{Binding, MutableBound};

pub struct AssetStorage<T> {
    assets: HashMap<PathBuf, Binding<Option<T>>>,
}

impl<T: Clone + PartialEq + Send + Sync + 'static> AssetStorage<T> {
    pub fn new() -> Self {
        Self {
            assets: HashMap::new(),
        }
    }

    pub fn get_asset(&mut self, key: impl Into<PathBuf>) -> &Binding<Option<T>> {
        let key = key.into();
        if self.assets.contains_key(&key) {
            self.assets.get(&key).unwrap()
        } else {
            // Insert new asset if it doesn't exist yet.
            self.assets.insert(key.clone(), Binding::new(None));
            self.assets.get(&key).unwrap()
        }
    }

    pub fn set_asset(&mut self, key: impl Into<PathBuf>, asset: T) {
        let key = key.into();
        if self.assets.contains_key(&key) {
            let stored_asset = self.assets.get(&key).unwrap();
            stored_asset.set(Some(asset));
        } else {
            self.assets.insert(key, Binding::new(Some(asset)));
        }
    }
}
