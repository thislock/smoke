use crate::{AtomicString, FileSystemLoader};

use super::asset_loader::{AssetLoader, AssetError};
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::{Arc, RwLock};

#[derive(Clone, Copy)]
pub enum ImageType {
  Png,
  Jpeg,
  Webm,
}

#[derive(Clone, Copy)]
pub struct Image {
  img_type: ImageType,
}

#[derive(Clone)]
pub enum FileData {
  TxtData(AtomicString),
  ImgData(Image),
}

pub struct AssetManager {
  loader: Arc<dyn AssetLoader>,
  cache: RwLock<HashMap<String, FileData>>,
}

fn program_directory() -> PathBuf {
  let exe = std::env::current_exe().expect("Failed to get current exe path");
  exe
    .parent()
    .expect("Why the fuck did you put this in the root directory? move it.")
    .to_path_buf()
}

impl AssetManager {
  pub fn new_local_filesystem() -> Self {
    Self {
      loader: Arc::new(FileSystemLoader::new(program_directory())),
      cache: RwLock::new(HashMap::new()),
    }
  }

  /// Load an asset, with caching
  pub async fn get(&self, path: &str) -> Result<FileData, AssetError> {
    // Check cache first
    if let Some(file) = self.cache.read().unwrap().get(path) {
      return Ok(file.clone());
    }

    // Otherwise load and cache
    let data = self.loader.load(path).await?;
    self
      .cache
      .write()
      .unwrap()
      .insert(path.to_string(), data.clone());

    Ok(data)
  }

  /// Clears cache (useful for hot reload or reloading shaders)
  pub fn clear_cache(&self) {
    self.cache.write().unwrap().clear();
  }
}
