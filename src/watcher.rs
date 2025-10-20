use crate::config::CONFIG;
use arboard::Clipboard;
use std::borrow::Cow;
use std::sync::mpsc::{Receiver, channel};
use std::thread::{sleep, spawn};

#[derive(Debug)]
pub struct Watcher {
  pub receiver: Receiver<String>,
}

impl Watcher {
  pub fn new() -> Self {
    let (sender, receiver) = channel();

    spawn(move || {
      let mut clipboard = Clipboard::new().unwrap();
      let mut last = clipboard.get_text().unwrap_or_default();

      loop {
        if let Ok(mut text) = clipboard.get_text()
          && text != last
        {
          last.clone_from(&text);
          transform(&mut text);
          let _ = sender.send(text);
        }

        sleep(CONFIG.watcher_interval());
      }
    });

    Self { receiver }
  }
}

fn transform(text: &mut String) {
  text.remove_matches(char::from(0));

  for regex in &CONFIG.regex {
    while regex.is_match(text) {
      let cow = regex.replace_all(text);
      if let Cow::Owned(inner) = cow {
        *text = inner;
      }
    }
  }

  for (key, value) in &CONFIG.replace {
    *text = text.replace(key, value);
  }
}
