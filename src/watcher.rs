use crate::config::CONFIG;
use crate::regex as re;
use arboard::Clipboard;
use std::sync::Arc;
use std::sync::atomic::AtomicBool;
use std::sync::atomic::Ordering::Relaxed;
use std::sync::mpsc::{Receiver, channel};
use std::thread::{sleep, spawn};

#[derive(Debug)]
pub struct Watcher {
  pub receiver: Receiver<String>,
  enabled: Arc<AtomicBool>,
}

impl Watcher {
  pub fn new() -> Self {
    let (sender, receiver) = channel();
    let enabled = Arc::new(AtomicBool::new(true));

    spawn({
      let enabled = Arc::clone(&enabled);
      move || {
        let mut clipboard = Clipboard::new().unwrap();
        let mut last = clipboard.get_text().unwrap_or_default();

        loop {
          if enabled.load(Relaxed)
            && let Ok(mut text) = clipboard.get_text()
            && text != last
          {
            last.clone_from(&text);
            transform(&mut text);

            let _ = sender.send(text);
          }

          sleep(CONFIG.watcher.interval());
        }
      }
    });

    Self { receiver, enabled }
  }

  pub fn enabled(&self) -> bool {
    self.enabled.load(Relaxed)
  }

  pub fn toggle(&self) {
    self.enabled.fetch_not(Relaxed);
  }
}

fn transform(text: &mut String) {
  while text.contains('\n') {
    *text = re::LINEBREAK.replace_all(text, "").to_string();
  }

  text.remove_matches(char::from(0));
}
