use regex::RegexBuilder;
use serde::{Deserialize, Deserializer};
use std::borrow::Cow;
use std::result::Result as StdResult;

#[derive(Deserialize)]
pub struct Regex {
  #[serde(alias = "pat", alias = "pattern")]
  inner: RegexInner,
  #[serde(alias = "replacement")]
  rep: String,
}

impl Regex {
  pub fn is_match(&self, haystack: &str) -> bool {
    self.inner.0.is_match(haystack)
  }

  pub fn replace_all<'a>(&self, haystack: &'a str) -> Cow<'a, str> {
    self.inner.0.replace_all(haystack, &self.rep)
  }
}

struct RegexInner(regex::Regex);

impl<'de> Deserialize<'de> for RegexInner {
  fn deserialize<D>(deserializer: D) -> StdResult<Self, D::Error>
  where
    D: Deserializer<'de>,
  {
    use serde::de::Error;

    let pattern = String::deserialize(deserializer)?;
    let regex = RegexBuilder::new(&pattern)
      .unicode(true)
      .build()
      .map_err(|_| D::Error::custom(format!("invalid regex: {pattern}")))?;

    Ok(Self(regex))
  }
}
