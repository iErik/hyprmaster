use std::cmp::Ordering;

#[derive(PartialEq, Debug)]
pub struct Semver {
  pub major: u8,
  pub minor: u8,
  pub patch: u16,
}

impl Semver {
  pub fn new(version: &str) -> Self {
    let mut ite = version.split('.');

    Self {
      major: ite.nth(0).unwrap_or("0").parse().unwrap_or(0),
      minor: ite.nth(0).unwrap_or("0").parse().unwrap_or(0),
      patch: ite.nth(0).unwrap_or("0").parse().unwrap_or(0),
    }
  }

  pub fn default() -> Self {
    Self {
      major: 0,
      minor: 0,
      patch: 0,
    }
  }
}

impl Eq for Semver {}

impl PartialOrd for Semver {
  fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
    Some(self.cmp(&other))
  }
}

impl Ord for Semver {
  fn cmp(&self, other: &Self) -> Ordering {
    if self.major != other.major {
      return self.major.cmp(&other.major)
    }

    if self.minor != other.minor {
      return self.minor.cmp(&other.minor)
    }

    if self.patch != other.patch {
      return self.patch.cmp(&other.patch)
    }

    Ordering::Equal
  }
}

