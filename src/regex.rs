use regex::Regex;
use std::sync::LazyLock;

macro_rules! re {
  ($p:literal) => {{ LazyLock::new(|| Regex::new($p).unwrap()) }};
}

pub static LINEBREAK: LazyLock<Regex> = re!(r"\n+\s?");
