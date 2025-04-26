use crate::config::CONFIG;
use crate::paragraph::{Paragraph, ParagraphPlacement};
use anyhow::Result;
use std::fs::OpenOptions;
use std::io::Write;

pub struct Cache(Vec<Entry>);

impl Cache {
  pub fn new() -> Self {
    let capacity = CONFIG.cache.capacity.get();
    Self(Vec::with_capacity(capacity))
  }

  pub fn write(&mut self) -> Result<()> {
    if !self.0.is_empty() {
      let path = &CONFIG.output.path;
      let mut text = String::with_capacity(120_000);
      for entry in self.0.drain(..) {
        match entry {
          Entry::Paragraph(paragraph) => {
            text.push_str(&paragraph.content);
          }
          Entry::Raw(string) => {
            text.push_str(string.as_ref());
          }
        }
      }

      OpenOptions::new()
        .append(true)
        .create(true)
        .open(path)?
        .write_all(text.as_bytes())?;
    }

    Ok(())
  }

  fn check_capacity(&mut self) -> Result<()> {
    let capacity = CONFIG.cache.capacity.get();
    if self.0.len() >= capacity.saturating_sub(1) {
      self.write()?;
    }

    Ok(())
  }

  pub fn raw(&mut self, text: &str) -> Result<()> {
    self.check_capacity()?;
    self.0.push(Entry::Raw(format!("\n\n{text}")));

    Ok(())
  }

  pub fn paragraph(&mut self, paragraph: &Paragraph) -> Result<()> {
    self.check_capacity()?;
    match paragraph.placement {
      ParagraphPlacement::After => {
        self.0.push(Entry::Paragraph(paragraph.clone()));
      }
      ParagraphPlacement::Before => {
        let last = self.0.pop();
        self.0.push(Entry::Paragraph(paragraph.clone()));

        if let Some(last) = last {
          self.0.push(last);
        }
      }
    }

    Ok(())
  }

  pub fn clear(&mut self) {
    self.0.clear();
  }

  pub fn pop(&mut self) {
    self.0.pop();
  }
}

enum Entry {
  Paragraph(Paragraph),
  Raw(String),
}
