mod interfaces;
mod dconf;
mod utils;
mod api;

//pub use objects::*;

pub mod apps {
  pub use super::interfaces::apps::*;
}

pub mod icons {
  pub use super::interfaces::icons::*;
}

pub mod tablet {
  pub use super::interfaces::tablet::*;
}
