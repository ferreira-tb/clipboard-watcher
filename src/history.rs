use crate::config::CONFIG;
use crate::paragraph::{Paragraph, ParagraphPlacement};
use ratatui::style::Stylize;
use ratatui::text::Text;
use ratatui::widgets::ListItem;
use std::collections::VecDeque;
use std::sync::atomic::AtomicU32;
use std::sync::atomic::Ordering::Relaxed;

static CURRENT: AtomicU32 = AtomicU32::new(1);

pub struct History {
  queue: VecDeque<Entry>,
}

impl History {
  pub fn new() -> Self {
    let capacity = CONFIG.history.capacity.get();
    History {
      queue: VecDeque::with_capacity(capacity),
    }
  }

  fn check_capacity(&mut self) {
    let capacity = CONFIG.history.capacity.get();
    while self.queue.len() >= capacity {
      self.queue.pop_front();
    }
  }

  pub fn raw(&mut self, text: &str) {
    self.check_capacity();
    let id = CURRENT.fetch_add(1, Relaxed);
    self
      .queue
      .push_back(Entry::raw(id, truncate(text)));
  }

  pub fn paragraph(&mut self, paragraph: &Paragraph) {
    self.check_capacity();
    match paragraph.placement {
      ParagraphPlacement::After => {
        self
          .queue
          .push_back(Entry::Paragraph(paragraph.clone()));
      }
      ParagraphPlacement::Before => {
        let last = self.queue.pop_back();
        self
          .queue
          .push_back(Entry::Paragraph(paragraph.clone()));

        if let Some(last) = last {
          self.queue.push_back(last);
        }
      }
    }
  }

  pub fn clear(&mut self) {
    self.queue.clear();
  }

  pub fn pop(&mut self) {
    self.queue.pop_back();
  }

  pub fn values(&self) -> impl Iterator<Item = &Entry> {
    self.queue.iter()
  }
}

pub enum Entry {
  Paragraph(Paragraph),
  Raw(u32, String),
}

impl Entry {
  fn raw(id: u32, content: &str) -> Self {
    Self::Raw(id, content.to_owned())
  }
}

impl<'a> From<&'a Entry> for ListItem<'a> {
  fn from(entry: &'a Entry) -> Self {
    match entry {
      Entry::Paragraph(paragraph) => {
        let text = if let Some(display) = paragraph.display.as_deref() {
          Text::from(truncate(display))
        } else {
          let width = CONFIG.history.width.get();
          let span = "-".repeat(width).bold().light_blue();
          Text::from(span)
        };

        ListItem::new(text)
      }
      Entry::Raw(id, content) => {
        let mut span = format!("({id})").bold();
        if id % 2 == 0 {
          span = span.magenta();
        } else {
          span = span.yellow();
        }

        let mut text = Text::from(span);
        text.push_span(" ");
        text.push_span(content.as_str());

        ListItem::new(text)
      }
    }
  }
}

fn truncate(s: &str) -> &str {
  let s = s.trim();
  let width = CONFIG.history.width.get();
  match s.char_indices().nth(width) {
    Some((idx, _)) => &s[..idx],
    None => s,
  }
}
