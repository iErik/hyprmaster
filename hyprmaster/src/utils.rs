extern crate unicode_normalization;
use unicode_normalization::{
  char::is_combining_mark,
  UnicodeNormalization
};

pub fn unaccent<T: AsRef<str>>(input: T) -> String {
  input
    .as_ref()
    .nfd()
    .filter(|c| !is_combining_mark(*c))
    .collect()
}

pub fn matches(target: &str, query: &str) -> bool {
  let query: String = unaccent(query);
  let query: Vec<String> = query
    .split(' ')
    .map(|s| s.to_lowercase())
    .collect();

  let target: String = unaccent(target);
  let target: Vec<String> = target
    .split(' ')
    .map(|s| s.to_lowercase())
    .collect();

  for word in query {
    let occurs = target.iter().any(|w| w.contains(&word));

    if !occurs { return false }
  }

  true
}
