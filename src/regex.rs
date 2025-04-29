use derive_more::Deref;
use regex::{Regex as RegexInner, RegexBuilder};
use serde::{Deserialize, Deserializer};
use std::result::Result as StdResult;

#[derive(Deref)]
pub struct Regex(RegexInner);

impl<'de> Deserialize<'de> for Regex {
  fn deserialize<D>(deserializer: D) -> StdResult<Self, D::Error>
  where
    D: Deserializer<'de>,
  {
    use serde::de::Error;

    let pattern = String::deserialize(deserializer)?;
    let regex = RegexBuilder::new(&pattern)
      .unicode(true)
      .build()
      .map_err(|_| D::Error::custom(format!("invalid regex pattern: {pattern}")))?;

    Ok(Self(regex))
  }
}
