use crate::config::CONFIG;
use crate::paragraph::{Paragraph, ParagraphPlacement};
use anyhow::Result;
use std::fs::{File, OpenOptions};
use std::io::{BufRead, Read, Write};

pub struct Cache {
  entries: Vec<Entry>,
  loc: usize,
}

impl Cache {
  pub fn new() -> Self {
    let capacity = CONFIG.cache.capacity.get();
    Self {
      entries: Vec::with_capacity(capacity),
      loc: loc(),
    }
  }

  pub fn write(&mut self) -> Result<()> {
    if !self.entries.is_empty() {
      let path = &CONFIG.output.path;
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
        .read(true)
        .open(path)?;

      file.write_all(buf.as_bytes())?;
      file.flush()?;
      file.sync_all()?;

      buf.clear();
      file.read_to_string(&mut buf)?;
      self.loc = buf.lines().count();
    }

    Ok(())
  }

  fn check_capacity(&mut self) -> Result<()> {
    let capacity = CONFIG.cache.capacity.get();
    if self.entries.len() >= capacity.saturating_sub(1) {
      self.write()?;
    }

    Ok(())
  }

  pub fn raw(&mut self, text: &str) -> Result<()> {
    self.check_capacity()?;
    self
      .entries
      .push(Entry::Raw(format!("\n\n{text}")));

    Ok(())
  }

  pub fn paragraph(&mut self, paragraph: &Paragraph) -> Result<()> {
    self.check_capacity()?;
    match paragraph.placement {
      ParagraphPlacement::After => {
        self
          .entries
          .push(Entry::Paragraph(paragraph.clone()));
      }
      ParagraphPlacement::Before => {
        let last = self.entries.pop();
        self
          .entries
          .push(Entry::Paragraph(paragraph.clone()));

        if let Some(last) = last {
          self.entries.push(last);
        }
      }
    }

    Ok(())
  }

  pub fn estimated_loc(&self) -> usize {
    let in_cache = self.entries.len().saturating_mul(2);
    self.loc.saturating_add(in_cache)
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
}

fn loc() -> usize {
  let loc: Result<usize> = try {
    File::open_buffered(&CONFIG.output.path)?
      .lines()
      .count()
  };

  loc.unwrap_or(0)
}

enum Entry {
  Paragraph(Paragraph),
  Raw(String),
}
