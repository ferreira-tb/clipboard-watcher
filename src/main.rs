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
        self.flush()?;
      }
      KeyCode::Char('l') => {
        self.cache.update_loc();
      }
      KeyCode::Char('p') => {
        Config::write_default()?;
      }
      KeyCode::Char('q') => {
        self.flush()?;
        self.exit();
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
            self.flush()?;
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
      KeyCode::Enter => {
        self.toggle()?;
      }
      _ => {}
    }

    Ok(())
  }

  fn collect(&mut self) -> Result<()> {
    if self.watcher.enabled() {
      for text in self.watcher.receiver.try_iter() {
        if !CONFIG.is_filtered(&text) {
          self.cache.raw(&text)?;
          self.history.raw(&text);
        }
      }
    }

    Ok(())
  }

  fn exit(&mut self) {
    self.exit = true;
  }

  fn flush(&mut self) -> Result<()> {
    self.cache.write()?;
    self.history.clear();
    Ok(())
  }

  fn toggle(&mut self) -> Result<()> {
    if self.watcher.enabled() {
      self.flush()?;
    }

    self.watcher.toggle();

    Ok(())
  }
}

impl Widget for &App {
  fn render(self, area: Rect, buf: &mut Buffer) {
    let title = Line::from(" Clipboard Watcher ".bold());
    let path = Line::from(format!(" {} ", CONFIG.path().display()));

    let block = Block::bordered()
      .title(title.centered())
      .title(status_line(self).left_aligned())
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

fn status_line(app: &App) -> Line<'_> {
  if app.watcher.enabled() {
    Line::from(" ON ".bold().green())
  } else {
    Line::from(" OFF ".bold().red())
  }
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

  let diff = max.saturating_sub(curr);
  if diff == 0 {
    Line::from(loc.bold().red())
  } else if (1..=100).contains(&diff) {
    Line::from(loc.bold().light_red())
  } else {
    Line::from(loc.bold())
  }
}
