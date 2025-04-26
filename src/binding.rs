use crate::paragraph::Paragraph;
use derive_more::Deref;
use serde::{Deserialize, Deserializer};
use std::collections::HashMap;
use std::result::Result as StdResult;

#[derive(Default, Deref)]
pub struct BindingTable(HashMap<u8, Paragraph>);

impl<'de> Deserialize<'de> for BindingTable {
  fn deserialize<D>(deserializer: D) -> StdResult<Self, D::Error>
  where
    D: Deserializer<'de>,
  {
    Ok(Self(
      Vec::<Binding>::deserialize(deserializer)?
        .into_iter()
        .map(|binding| (binding.key, binding.paragraph))
        .collect(),
    ))
  }
}

#[derive(Deserialize)]
struct Binding {
  key: u8,
  #[serde(flatten)]
  paragraph: Paragraph,
}
