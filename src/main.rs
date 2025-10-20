#![feature(file_buffered, string_remove_matches, try_blocks)]

mod binding;
mod cache;
mod config;
mod history;
mod paragraph;
mod regex;
mod watcher;

use anyhow::Result;
use cache::Cache;
use config::{CONFIG, Config};
use crossterm::event::{self, Event, KeyCode, KeyEvent, KeyEventKind};
use history::History;
use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use ratatui::style::Stylize;
use ratatui::symbols::border;
use ratatui::text::Line;
use ratatui::widgets::{Block, List, ListDirection, Widget};
use ratatui::{DefaultTerminal, Frame};
use watcher::Watcher;

fn main() -> Result<()> {
  let mut app = App::new();
  let mut terminal = ratatui::init();
  let result = app.run(&mut terminal);
  ratatui::restore();
  result
}

struct App {
  cache: Cache,
  history: History,
  watcher: Watcher,
  exit: bool,
}

impl App {
  fn new() -> Self {
    Self {
      cache: Cache::new(),
      history: History::new(),
      watcher: Watcher::new(),
      exit: false,
    }
  }

  fn run(&mut self, terminal: &mut DefaultTerminal) -> Result<()> {
    while !self.exit {
      self.collect()?;
      terminal.draw(|frame| self.draw(frame))?;

      if event::poll(CONFIG.poll_interval())?
        && let Event::Key(event) = event::read()?
        && event.kind == KeyEventKind::Press
      {
        self.on_key(event)?;
      }
    }

    Ok(())
  }

  fn draw(&self, frame: &mut Frame) {
    frame.render_widget(self, frame.area());
  }

  fn on_key(&mut self, event: KeyEvent) -> Result<()> {
    match event.code {
      KeyCode::Char('f') => {
        flush(self).call()?;
      }

      KeyCode::Char('l') => {
        self.cache.update_loc();
      }
      KeyCode::Char('p') => {
        Config::write_default()?;
      }
      KeyCode::Char('q') => {
        flush(self).call()?;
        self.exit();
      }
      KeyCode::Char('w') => {
        flush(self).clear_history(false).call()?;
      }
      KeyCode::Char('x') => {
        self.exit();
      }
      KeyCode::Char(value @ '0'..='9') => {
        if let Some(digit) = value.to_digit(10)
          && let Ok(num) = u8::try_from(digit)
          && let Some(paragraph) = CONFIG.bindings.get(&num)
        {
          self.cache.paragraph(paragraph)?;
          self.history.paragraph(paragraph);
          if paragraph.flush {
            flush(self).call()?;
          }
        }
      }

      KeyCode::Backspace => {
        self.cache.pop();
        self.history.pop();
      }
      KeyCode::Delete => {
        self.cache.clear();
        self.history.clear();
      }
      _ => {}
    }

    Ok(())
  }

  fn collect(&mut self) -> Result<()> {
    for text in self.watcher.receiver.try_iter() {
      if !CONFIG.is_filtered(&text) {
        self.cache.raw(&text)?;
        self.history.raw(&text);
      }
    }

    Ok(())
  }

  fn exit(&mut self) {
    self.exit = true;
  }
}

impl Widget for &App {
  fn render(self, area: Rect, buf: &mut Buffer) {
    let title = Line::from(" Clipboard Watcher ".bold());
    let path = Line::from(format!(" {} ", CONFIG.path().display()));

    let block = Block::bordered()
      .title(title.centered())
      .title(cache_line(self).right_aligned())
      .title_bottom(path.centered())
      .title_bottom(loc_line(self).left_aligned())
      .border_set(border::THICK);

    List::new(self.history.values())
      .block(block)
      .direction(ListDirection::TopToBottom)
      .scroll_padding(3)
      .render(area, buf);
  }
}

#[bon::builder]
fn flush(
  #[builder(start_fn)] app: &mut App,
  #[builder(default = true)] clear_history: bool,
) -> Result<()> {
  app.cache.write()?;

  if clear_history {
    app.history.clear();
  }

  Ok(())
}

fn cache_line(app: &App) -> Line<'_> {
  let len = app.cache.len();
  let cap = CONFIG.cache_capacity();
  Line::from(format!(" Cache: {len} / {cap} ").bold())
}

fn loc_line(app: &App) -> Line<'_> {
  let curr = app.cache.estimated_loc();
  let max = CONFIG.max_loc();
  let loc = format!(" {curr} / {max} ");

  if curr >= max { Line::from(loc.bold().red()) } else { Line::from(loc.bold()) }
}
