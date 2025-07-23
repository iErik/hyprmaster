use std::process::{Command, Output, Stdio, ExitStatus};
use std::error::Error;

use crate::utils::semver::Semver;

pub struct OpenTabletDriver {
  version: Semver
}

impl OpenTabletDriver {
  pub fn new () -> Self {
    let version = Command::new("otd")
      .arg("--version")
      .output();

    let version = match version {
      Ok(Output { stdout, ..}) => Semver::new(
        std::str::from_utf8(&stdout).unwrap_or("0.0.0")),
      Err(_) => Semver::new("0.0.0")
    };

    Self { version }
  }

  pub fn apply_preset(&self, preset_name: &str)
    -> std::io::Result<ExitStatus>
  {
    let mut cmd = Command::new("otd");
    println!("version: {:#?}", self.version);

    if self.version.minor >= 7 {
      cmd.arg("apply-preset");
    } else {
      cmd.arg("applypreset");
    }

    cmd.arg(preset_name)
      .stdout(Stdio::null())
      .spawn()?
      .wait()
  }

  fn save_preset(&self) {

  }

  fn set_display_area(&self) {

  }

  fn set_tablet_area(&self) {

  }

  fn set_relative_sensitivity(&self) {

  }

  fn set_tip_binding(&self) {

  }

  fn set_pen_binding(&self) {

  }

  fn set_aux_binding(&self) {

  }

  fn set_output_mode(&self) {

  }

  fn set_filters(&self) {

  }

  fn set_tools(&self) {

  }

  fn log(&self) {

  }

  fn get_all_settings(&self) {

  }

  fn get_area_settings(&self) {

  }

  fn get_sensitivity(&self) {

  }

  fn get_bindings(&self) {

  }

  fn get_output_mode(&self) {

  }

  fn get_filters(&self) {

  }

  fn get_tools(&self) {

  }

  fn get_device_string(&self) {

  }

  fn detect(&self) {

  }

  fn list_presets(&self) {

  }


  fn list_tablets(&self) {

  }

  fn list_output_modes(&self) {

  }

  fn list_filters(&self) {}

  fn list_bindings(&self) {}

  fn list_tools(&self) {}

  fn edit_settings(&self) { }

  fn diagnostics(&self) {}
}
