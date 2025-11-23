use async_trait::async_trait;
use tokio::sync::mpsc::{UnboundedReceiver, UnboundedSender, unbounded_channel};
use tokio::task;

use notify::{RecommendedWatcher, RecursiveMode, Watcher, EventKind};
use std::collections::HashMap;
use std::time::Duration;

use std::sync::{Arc, Mutex, RwLock};

use crate::FileData;

#[derive(Debug, Clone, Default)]
pub struct AtomicString {
  inner: Arc<RwLock<String>>,
}

impl AtomicString {
  pub fn new<S: Into<String>>(s: S) -> Self {
    Self {
      inner: Arc::new(RwLock::new(s.into())),
    }
  }

  pub fn get(&self) -> String {
    self.inner.read().unwrap().clone()
  }

  pub fn store<S: Into<String>>(&self, new_value: S) {
    *self.inner.write().unwrap() = new_value.into();
  }
}

impl From<String> for AtomicString {
  fn from(s: String) -> Self {
    Self::new(s)
  }
}

/// Generic error type for asset operations
#[derive(Debug)]
pub enum AssetError {
  NotFound(String),
  Io(std::io::Error),
  Network(reqwest::Error),
  Decode(String),
}

#[async_trait]
pub trait AssetLoader: Send + Sync {
  async fn load(&self, path: &str) -> Result<FileData, AssetError>;
}

#[derive(Debug, Clone)]
pub enum AssetEvent {
  Modified(String),
  Removed(String),
}

/// Local filesystem loader (for native targets)
pub struct FileSystemLoader {
  base_path: std::path::PathBuf,
  loaded_files: Mutex<HashMap<String, FileData>>,
  watchers: Option<(UnboundedSender<AssetEvent>, UnboundedReceiver<AssetEvent>)>,
}

impl FileSystemLoader {
  pub fn new(base_path: impl Into<std::path::PathBuf>) -> Self {
    Self {
      loaded_files: Mutex::new(HashMap::new()),
      base_path: base_path.into(),
      watchers: None,
    }
  }

  /// Start watching this loader's directory for hot reload events.
  pub fn watch(&mut self) {
    // if we're already watching this directory, don't add more watchers
    if let Some(watcher) = &self.watchers {
      if !watcher.0.is_closed() {
        return;
      }
    }
    let base_path = self.base_path.clone();
    self.watchers = Some(unbounded_channel::<AssetEvent>());
    let tx = self.watchers.as_mut().unwrap().0.clone();

    // spawn a tokio thread to sit around and wait for filesystem interupts
    task::spawn_blocking(move || {
      let mut watcher: RecommendedWatcher =
        notify::recommended_watcher(move |res: Result<notify::Event, notify::Error>| {
          match_methods(&tx, res);
        })
        .expect("Failed to create file watcher");

      watcher
        .watch(&base_path, RecursiveMode::Recursive)
        .expect("Failed to watch asset directory");

      // sit around forever so watcher isn't de allocated
      loop {
        std::thread::sleep(Duration::from_secs(10));
      }
    });
  }
}

fn match_methods(tx: &UnboundedSender<AssetEvent>, result: Result<notify::Event, notify::Error>) {
  match result {
    Ok(event) => {
      update_files(tx, event);
    }
    Err(e) => eprintln!("watch error: {:?}", e),
  }
}

fn update_files(tx: &UnboundedSender<AssetEvent>, event: notify::Event) {
  if let Some(path) = event.paths.first() {
    if let Some(filename) = path.file_name().and_then(|n| n.to_str()) {
      // TODO: store all read files as atomic strings and send them here.
      match event.kind {
        EventKind::Modify(_) => {
          let _ = tx.send(AssetEvent::Modified(filename.to_string()));
        }
        EventKind::Remove(_) => {
          let _ = tx.send(AssetEvent::Removed(filename.to_string()));
        }
        _ => {}
      }
    }
  }
}

#[async_trait]
impl AssetLoader for FileSystemLoader {
  async fn load(&self, path: &str) -> Result<FileData, AssetError> {
    let full_path = self.base_path.join(path);
    let file = tokio::fs::read_to_string(&full_path)
      .await
      .map_err(|e| AssetError::Io(e))
      .map(|a| FileData::TxtData(a.into()))?;

    self
      .loaded_files
      .lock()
      .map_err(|e| AssetError::Decode(e.to_string()))?
      .insert(
        full_path.to_str().unwrap().to_ascii_lowercase(),
        file.clone(),
      );

    return Ok(file);
  }
}

/// Web loader (for WASM or network builds) (do later i dont wanna)
pub struct WebLoader {
  base_url: String,
}

impl WebLoader {
  pub fn new(base_url: impl Into<String>) -> Self {
    Self {
      base_url: base_url.into(),
    }
  }
}

// #[async_trait]
// impl AssetLoader for WebLoader {
//   async fn load(&self, path: &str) -> Result<FileData, AssetError> {
//     let url = format!("{}/{}", self.base_url.trim_end_matches('/'), path);
//     let response = reqwest::get(&url).await.map_err(AssetError::Network)?;
//     let bytes = response.bytes().await.map_err(AssetError::Network)?;
//     Ok(bytes.to)
//   }
// }
