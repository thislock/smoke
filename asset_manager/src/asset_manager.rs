use super::asset_loader::{AssetLoader, AssetError};
use std::collections::HashMap;
use std::sync::{Arc, RwLock};

pub struct AssetManager {
  loader: Arc<dyn AssetLoader>,
  cache: RwLock<HashMap<String, Vec<u8>>>,
}

impl AssetManager {
  pub fn new(loader: Arc<dyn AssetLoader>) -> Self {
    Self {
      loader,
      cache: RwLock::new(HashMap::new()),
    }
  }

  /// Load an asset, with caching
  pub async fn get(&self, path: &str) -> Result<Arc<Vec<u8>>, AssetError> {
    // Check cache first
    if let Some(bytes) = self.cache.read().unwrap().get(path) {
      return Ok(Arc::new(bytes.clone()));
    }

    // Otherwise load and cache
    let data = self.loader.load(path).await?;
    self
      .cache
      .write()
      .unwrap()
      .insert(path.to_string(), data.clone());
    Ok(Arc::new(data))
  }

  /// Clears cache (useful for hot reload or reloading shaders)
  pub fn clear_cache(&self) {
    self.cache.write().unwrap().clear();
  }
}
