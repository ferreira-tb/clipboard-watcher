use crate::binding::BindingTable;
use crate::regex::Regex;
use anyhow::Result;
use derive_more::Deref;
use serde::Deserialize;
use std::collections::HashMap;
use std::fs::{self, File};
use std::io::Write;
use std::num::{NonZeroU64, NonZeroUsize};
use std::path::{Path, PathBuf};
use std::sync::LazyLock;
use std::time::Duration;
use walkdir::{DirEntry, WalkDir};

const FILENAME: &str = "clipboard.toml";
const DEFAULT_CONFIG: &str = include_str!("../clipboard.toml");

pub static CONFIG: LazyLock<Config> = LazyLock::new(Config::load);

#[derive(Default, Deserialize)]
pub struct Config {
  #[serde(default)]
  pub path: OutputPath,
  #[serde(default)]
  pub max_loc: MaxLoc,
  #[serde(default)]
  pub poll_interval: EventPollInterval,
  #[serde(default)]
  pub cache_capacity: CacheCapacity,
  #[serde(default)]
  pub history_capacity: HistoryCapacity,
  #[serde(default)]
  pub history_width: HistoryWidth,
  #[serde(default)]
  pub watcher_interval: WatcherInterval,
  #[serde(default)]
  pub bindings: BindingTable,

  #[serde(default)]
  pub filter: Vec<String>,
  #[serde(default)]
  pub replace: HashMap<String, String>,
  #[serde(default)]
  pub regex: Vec<Regex>,
}

impl Config {
  fn load() -> Self {
    let result: Result<Self> = try {
      let path = WalkDir::new(FILENAME)
        .into_iter()
        .flatten()
        .find(|entry| !entry.file_type().is_dir())
        .map(DirEntry::into_path);

      if let Some(path) = path {
        let contents = fs::read_to_string(&path)?;
        toml::from_str(&contents)?
      } else {
        Self::default()
      }
    };

    result.unwrap()
  }

  pub fn write_default() -> Result<()> {
    let mut file = File::create_new(FILENAME)?;
    file.write_all(DEFAULT_CONFIG.as_bytes())?;
    Ok(())
  }

  pub fn path(&self) -> &Path {
    &self.path.0
  }

  pub const fn max_loc(&self) -> usize {
    self.max_loc.0.get()
  }

  pub const fn poll_interval(&self) -> Duration {
    Duration::from_millis(self.poll_interval.0.get())
  }

  pub const fn cache_capacity(&self) -> usize {
    self.cache_capacity.0.get()
  }

  pub const fn history_capacity(&self) -> usize {
    self.history_capacity.0.get()
  }

  pub const fn history_width(&self) -> usize {
    self.history_width.0.get()
  }

  pub const fn watcher_interval(&self) -> Duration {
    Duration::from_millis(self.watcher_interval.0.get())
  }

  pub fn is_filtered(&self, text: &str) -> bool {
    self.filter.iter().any(|f| text.contains(f))
  }
}

#[derive(Clone, Copy, Deref, Deserialize)]
pub struct MaxLoc(NonZeroUsize);

impl Default for MaxLoc {
  fn default() -> Self {
    Self(unsafe { NonZeroUsize::new_unchecked(1500) })
  }
}

#[derive(Clone, Copy, Deref, Deserialize)]
pub struct EventPollInterval(NonZeroU64);

impl Default for EventPollInterval {
  fn default() -> Self {
    Self(unsafe { NonZeroU64::new_unchecked(25) })
  }
}

#[derive(Deref, Deserialize)]
pub struct OutputPath(PathBuf);

impl AsRef<Path> for OutputPath {
  fn as_ref(&self) -> &Path {
    self.0.as_path()
  }
}

impl Default for OutputPath {
  fn default() -> Self {
    Self(PathBuf::from("clipboard.md"))
  }
}

#[derive(Clone, Copy, Deref, Deserialize)]
pub struct CacheCapacity(NonZeroUsize);

impl Default for CacheCapacity {
  fn default() -> Self {
    Self(unsafe { NonZeroUsize::new_unchecked(100) })
  }
}

#[derive(Clone, Copy, Deref, Deserialize)]
pub struct HistoryCapacity(NonZeroUsize);

impl Default for HistoryCapacity {
  fn default() -> Self {
    Self(unsafe { NonZeroUsize::new_unchecked(45) })
  }
}

#[derive(Clone, Copy, Deref, Deserialize)]
pub struct HistoryWidth(NonZeroUsize);

impl Default for HistoryWidth {
  fn default() -> Self {
    Self(unsafe { NonZeroUsize::new_unchecked(80) })
  }
}

#[derive(Clone, Copy, Deref, Deserialize)]
pub struct WatcherInterval(NonZeroU64);

impl Default for WatcherInterval {
  fn default() -> Self {
    Self(unsafe { NonZeroU64::new_unchecked(10) })
  }
}
