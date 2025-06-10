use crate::config::CONFIG;
use crate::paragraph::{Paragraph, ParagraphPlacement};
use ratatui::style::Stylize;
use ratatui::text::Text;
use ratatui::widgets::ListItem;
use std::collections::VecDeque;
use std::num::NonZeroU32;

pub struct History {
  queue: VecDeque<Entry>,
  current: NonZeroU32,
}

impl History {
  pub fn new() -> Self {
    let capacity = CONFIG.history.capacity.get();
    History {
      queue: VecDeque::with_capacity(capacity),
      current: NonZeroU32::MIN,
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
    self
      .queue
      .push_back(Entry::raw(self.current, truncate(text)));

    self.current = self.current.saturating_add(1);
  }

  pub fn paragraph(&mut self, paragraph: &Paragraph) {
    use ParagraphPlacement::*;

    self.check_capacity();
    match paragraph.placement {
      After => {
        self
          .queue
          .push_back(Entry::Paragraph(paragraph.clone()));
      }
      Before | Replace => {
        let last = self.pop();
        self
          .queue
          .push_back(Entry::Paragraph(paragraph.clone()));

        if let Some(last) = last
          && let Before = paragraph.placement
        {
          if let Entry::Raw(_, text) = last {
            self.raw(&text);
          } else {
            self.queue.push_back(last);
          }
        }
      }
    }
  }

  pub fn clear(&mut self) {
    self.queue.clear();
    self.current = NonZeroU32::MIN;
  }

  pub fn pop(&mut self) -> Option<Entry> {
    self.queue.pop_back().inspect(|entry| {
      if matches!(entry, Entry::Raw(_, _)) && self.current > NonZeroU32::MIN {
        unsafe {
          let n = self.current.get().unchecked_sub(1);
          self.current = NonZeroU32::new_unchecked(n);
        }
      }
    })
  }

  pub fn values(&self) -> impl Iterator<Item = &Entry> {
    self.queue.iter()
  }
}

pub enum Entry {
  Paragraph(Paragraph),
  Raw(NonZeroU32, String),
}

impl Entry {
  fn raw(id: NonZeroU32, content: &str) -> Self {
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
        if id.get() % 2 == 0 {
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

fn truncate(text: &str) -> &str {
  let text = text.trim();
  let width = CONFIG.history.width.get();
  match text.char_indices().nth(width) {
    Some((idx, _)) => &text[..idx],
    None => text,
  }
}
