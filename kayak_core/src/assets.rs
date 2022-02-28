use resources::RefMut;
use std::{collections::HashMap, path::PathBuf};

use crate::{Binding, MutableBound};

// TODO: Consider aliasing Binding<Option<T>> to be more ergonomic (or maybe use a wrapper struct)
// pub type AssetRef<T> = Binding<Option<T>>;

pub struct AssetStorage<T> {
    assets: HashMap<PathBuf, Binding<Option<T>>>,
}

impl<T: Clone + PartialEq + Send + Sync + 'static> AssetStorage<T> {
    pub fn new() -> Self {
        Self {
            assets: HashMap::new(),
        }
    }

    pub fn get(&mut self, key: impl Into<PathBuf>) -> &Binding<Option<T>> {
        let key = key.into();
        if self.assets.contains_key(&key) {
            self.assets.get(&key).unwrap()
        } else {
            // Insert new asset if it doesn't exist yet.
            self.assets.insert(key.clone(), Binding::new(None));
            self.assets.get(&key).unwrap()
        }
    }

    pub fn set(&mut self, key: impl Into<PathBuf>, asset: T) {
        let key = key.into();
        if self.assets.contains_key(&key) {
            let stored_asset = self.assets.get(&key).unwrap();
            stored_asset.set(Some(asset));
        } else {
            self.assets.insert(key, Binding::new(Some(asset)));
        }
    }
}

/// A collection for storing assets in Kayak
///
/// This handles getting and setting assets in such a way as to allow them to
/// be bindable by widgets.
#[derive(Default)]
pub struct Assets {
    assets: resources::Resources,
}

impl Assets {
    /// Get a stored asset with the given asset key
    ///
    /// The type of the asset [T] must implement `Clone` and `PartialEq` so that a `Binding<Option<T>>`
    /// can be returned. By calling [bind](Self::bind) over the binding, you can react to all changes to
    /// the asset, including when it's added or removed.
    ///
    /// If no asset in storage matches both the asset key _and_ the asset type, a value of
    /// `Binding<None>` is returned. Again, binding to this value will allow you to detect when a matching
    /// asset is added to storage.
    ///
    /// # Arguments
    ///
    /// * `key`: The asset key
    ///
    pub fn get_asset<T: 'static + Send + Sync + Clone + PartialEq, K: Into<PathBuf>>(
        &mut self,
        key: K,
    ) -> Binding<Option<T>> {
        let mut asset_storage = self.get_mut::<T>();
        asset_storage.get(key).clone()
    }

    /// Stores an asset along with a key to access it
    ///
    /// # Arguments
    ///
    /// * `key`: The asset key
    /// * `asset`: The asset to store
    ///
    pub fn set_asset<T: 'static + Send + Sync + Clone + PartialEq, K: Into<PathBuf>>(
        &mut self,
        key: K,
        asset: T,
    ) {
        let mut asset_storage = self.get_mut::<T>();
        asset_storage.set(key, asset);
    }

    /// Get a mutable reference to the asset storage
    fn get_mut<T: 'static + Send + Sync + Clone + PartialEq>(&mut self) -> RefMut<AssetStorage<T>> {
        self.assets
            .entry()
            .or_insert_with(|| AssetStorage::<T>::new())
    }
}
