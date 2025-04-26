use serde::Deserialize;
use std::sync::Arc;

#[derive(Deserialize)]
pub struct Paragraph {
  pub content: Arc<str>,
  #[serde(default)]
  pub placement: ParagraphPlacement,
  #[serde(default)]
  pub flush: bool,
  #[serde(default)]
  pub display: Option<Arc<str>>,
}

impl Clone for Paragraph {
  fn clone(&self) -> Self {
    Self {
      content: Arc::clone(&self.content),
      placement: self.placement,
      flush: self.flush,
      display: self.display.as_ref().map(Arc::clone),
    }
  }
}

#[derive(Clone, Copy, Default, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum ParagraphPlacement {
  #[default]
  After,
  Before,
}
