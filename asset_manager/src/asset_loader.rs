use async_trait::async_trait;
use tokio::sync::mpsc::{UnboundedReceiver, UnboundedSender, unbounded_channel};
use tokio::task;

use notify::{RecommendedWatcher, RecursiveMode, Watcher, EventKind};
use std::time::Duration;

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
  async fn load(&self, path: &str) -> Result<Vec<u8>, AssetError>;
}

#[derive(Debug, Clone)]
pub enum AssetEvent {
  Modified(String),
  Removed(String),
}

/// Local filesystem loader (for native targets)
pub struct FileSystemLoader {
  base_path: std::path::PathBuf,
  watcher_tx: Option<UnboundedSender<AssetEvent>>,
}

impl FileSystemLoader {
  pub fn new(base_path: impl Into<std::path::PathBuf>) -> Self {
    Self {
      base_path: base_path.into(),
      watcher_tx: None,
    }
  }
  
  /// Start watching this loader's directory for hot reload events.
  pub fn watch(&mut self) -> UnboundedReceiver<AssetEvent> {
    let base_path = self.base_path.clone();
    let (tx, rx) = unbounded_channel::<AssetEvent>();
    self.watcher_tx = Some(tx.clone());

    // spawn a tokio thread to sit around and wait for filesystem interupts
    task::spawn_blocking(move || {
      let mut watcher: RecommendedWatcher = notify::recommended_watcher(move | res: Result<notify::Event, notify::Error> | match res {
        Ok(event) => {
          if let Some(path) = event.paths.first() {
            if let Some(filename) = path.file_name().and_then(|n| n.to_str()) {
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
        Err(e) => eprintln!("watch error: {:?}", e),
      })
      .expect("Failed to create file watcher");

      watcher
        .watch(&base_path, RecursiveMode::Recursive)
        .expect("Failed to watch asset directory");

      loop {
        std::thread::sleep(Duration::from_secs(10));
      }
    });

    rx
  }
}

#[async_trait]
impl AssetLoader for FileSystemLoader {
  async fn load(&self, path: &str) -> Result<Vec<u8>, AssetError> {
    let full_path = self.base_path.join(path);
    tokio::fs::read(&full_path)
      .await
      .map_err(|e| AssetError::Io(e))
  }
}

/// Web loader (for WASM or network builds)
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

#[async_trait]
impl AssetLoader for WebLoader {
  async fn load(&self, path: &str) -> Result<Vec<u8>, AssetError> {
    let url = format!("{}/{}", self.base_url.trim_end_matches('/'), path);
    let response = reqwest::get(&url).await.map_err(AssetError::Network)?;
    let bytes = response.bytes().await.map_err(AssetError::Network)?;
    Ok(bytes.to_vec())
  }
}
