use std::env::var;
use std::path::PathBuf;

pub struct XDGPaths {
  pub home_dir: PathBuf,
  pub data_home: PathBuf,
  pub config_dir: PathBuf,
}

impl XDGPaths {
  pub fn new() -> Self {
    let home_dir =
      std::env::var("HOME").unwrap_or("~".to_string());
    let data_home = std::env::var("XDG_DATA_HOME")
      .unwrap_or(format!("{home_dir}/.local/share"));

    let config_dir = std::env::var("XDG_CONFIG_HOME")
      .unwrap_or(format!("{home_dir}/.config"));

    Self {
      home_dir: home_dir.into(),
      data_home: data_home.into(),
      config_dir: config_dir.into(),
    }
  }
}
