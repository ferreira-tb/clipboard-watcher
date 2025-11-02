use crate::config::CONFIG;
use crate::paragraph::{Paragraph, ParagraphPlacement};
use anyhow::Result;
use std::fs::{File, OpenOptions};
use std::io::{BufRead, Write};

pub struct Cache {
  entries: Vec<Entry>,
  loc: usize,
}

impl Cache {
  pub fn new() -> Self {
    let capacity = CONFIG.cache_capacity();
    Self {
      entries: Vec::with_capacity(capacity),
      loc: loc(),
    }
  }

  pub fn write(&mut self) -> Result<()> {
    if !self.entries.is_empty() {
      let path = CONFIG.path();
      let mut buf = String::with_capacity(150_000);
      for entry in self.entries.drain(..) {
        match entry {
          Entry::Paragraph(paragraph) => {
            buf.push_str(&paragraph.content);
          }
          Entry::Raw(string) => {
            buf.push_str(string.as_ref());
          }
        }
      }

      let mut file = OpenOptions::new()
        .append(true)
        .create(true)
        .open(path)?;

      file.write_all(buf.as_bytes())?;
      file.flush()?;
      file.sync_all()?;
    }

    self.update_loc();

    Ok(())
  }

  fn check_capacity(&mut self) -> Result<()> {
    let capacity = CONFIG.cache_capacity();
    if self.entries.len() >= capacity.saturating_sub(1) {
      self.write()?;
    }

    Ok(())
  }

  pub fn raw(&mut self, text: &str) -> Result<()> {
    self.check_capacity()?;
    let text = format!("\n\n{}", text.trim());
    self.entries.push(Entry::Raw(text));
    Ok(())
  }

  pub fn paragraph(&mut self, paragraph: &Paragraph) -> Result<()> {
    use ParagraphPlacement::*;

    self.check_capacity()?;
    match paragraph.placement {
      After => {
        self
          .entries
          .push(Entry::Paragraph(paragraph.clone()));
      }
      Before | Replace => {
        let last = self.entries.pop();
        self
          .entries
          .push(Entry::Paragraph(paragraph.clone()));

        if let Some(last) = last
          && let Before = paragraph.placement
        {
          self.entries.push(last);
        }
      }
    }

    Ok(())
  }

  pub fn estimated_loc(&self) -> usize {
    let in_cache = self.entries.len().saturating_mul(2);
    let mut loc = self.loc.saturating_add(in_cache);

    if loc.is_multiple_of(2) {
      loc = loc.saturating_add(1);
    }

    loc
  }

  pub fn update_loc(&mut self) {
    self.loc = loc();
  }

  pub fn clear(&mut self) {
    self.entries.clear();
  }

  pub fn pop(&mut self) {
    self.entries.pop();
  }

  pub fn len(&self) -> usize {
    self.entries.len()
  }
}

fn loc() -> usize {
  let loc: Result<usize> = try {
    File::open_buffered(CONFIG.path())?
      .lines()
      .count()
  };

  loc.unwrap_or(0)
}

enum Entry {
  Paragraph(Paragraph),
  Raw(String),
}
