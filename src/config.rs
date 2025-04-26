use crate::binding::BindingTable;
use anyhow::{Result, anyhow};
use derive_more::Deref;
use serde::Deserialize;
use std::fs::{self, File};
use std::io::Write;
use std::num::{NonZeroU64, NonZeroUsize};
use std::path::PathBuf;
use std::sync::LazyLock;
use std::time::Duration;
use walkdir::WalkDir;

const FILENAME: &str = "clipboard.toml";
const DEFAULT_CONFIG: &str = include_str!("../clipboard.toml");

pub static CONFIG: LazyLock<Config> = LazyLock::new(Config::load);

#[derive(Deserialize)]
pub struct Config {
  pub app: AppConfig,
  pub output: OutputConfig,
  pub cache: CacheConfig,
  pub history: HistoryConfig,
  pub watcher: WatcherConfig,
  pub bindings: BindingTable,
}

impl Config {
  fn load() -> Self {
    let result: Result<Self> = try {
      let path = WalkDir::new(FILENAME)
        .into_iter()
        .flatten()
        .find(|entry| !entry.file_type().is_dir())
        .ok_or_else(|| anyhow!("config file not found"))?
        .into_path();

      let contents = fs::read_to_string(&path)?;
      let config = toml::from_str(&contents)?;
      config
    };

    result.unwrap()
  }

  pub fn write_default() -> Result<()> {
    let mut file = File::create_new(FILENAME)?;
    file.write_all(DEFAULT_CONFIG.as_bytes())?;
    Ok(())
  }
}

#[derive(Deserialize)]
pub struct AppConfig {
  #[serde(default)]
  poll_interval: EventPollInterval,
}

impl AppConfig {
  pub fn poll_interval(&self) -> Duration {
    Duration::from_millis(self.poll_interval.get())
  }
}

#[derive(Clone, Copy, Deref, Deserialize)]
pub struct EventPollInterval(NonZeroU64);

impl Default for EventPollInterval {
  fn default() -> Self {
    Self(unsafe { NonZeroU64::new_unchecked(25) })
  }
}

#[derive(Deserialize)]
pub struct OutputConfig {
  pub path: PathBuf,
}

#[derive(Deserialize)]
pub struct CacheConfig {
  #[serde(default)]
  pub capacity: CacheCapacity,
}

#[derive(Clone, Copy, Deref, Deserialize)]
pub struct CacheCapacity(NonZeroUsize);

impl Default for CacheCapacity {
  fn default() -> Self {
    Self(unsafe { NonZeroUsize::new_unchecked(100) })
  }
}

#[derive(Deserialize)]
pub struct HistoryConfig {
  #[serde(default)]
  pub capacity: HistoryCapacity,
  #[serde(default)]
  pub width: HistoryWidth,
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

#[derive(Deserialize)]
pub struct WatcherConfig {
  #[serde(default)]
  interval: WatcherInterval,
}

impl WatcherConfig {
  pub fn interval(&self) -> Duration {
    Duration::from_millis(self.interval.get())
  }
}

#[derive(Clone, Copy, Deref, Deserialize)]
pub struct WatcherInterval(NonZeroU64);

impl Default for WatcherInterval {
  fn default() -> Self {
    Self(unsafe { NonZeroU64::new_unchecked(10) })
  }
}
