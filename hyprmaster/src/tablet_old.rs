use std::fs::DirEntry;
use std::path;
use std::collections::HashMap;

use ini::configparser::ini::Ini;

use crate::xdg::XDGPaths;
use crate::ui::TabletUIState;



type SectionMap = HashMap<String, Option<String>>;

#[derive(Debug)]
pub struct TabletPreset {
  name: String,
  path: path::PathBuf,
}

pub struct TabletMaster<'a> {
  state: TabletUIState<'a>,
  presets: Vec<TabletPreset>,
  config: Ini,
}

impl<'a> TabletMaster<'a> {
  pub fn new(state: TabletUIState<'a>) -> Self {
    let mut config = Ini::new();
    let mut config_dir = XDGPaths::new().config_dir;

    config_dir.push("hyprmaster/tablet.conf");
    _ = config.load(config_dir.to_str().unwrap());

    Self {
      state,
      presets: find_presets().unwrap_or(vec![]),
      config,
    }
  }

  pub fn bound_presets(&self) -> Option<&SectionMap> {
    self
      .config
      .get_map_ref()
      .get("preset bindings")
  }

  pub fn bind_preset(
    &mut self,
    app: &str,
    preset: &str,
  ) -> Option<Option<String>> {
    self.config.set(
      "Preset Bindings",
      app,
      Some(preset.to_string()),
    )
  }
}



pub fn find_presets() -> Option<Vec<TabletPreset>> {
  let mut presets_dir = XDGPaths::new().config_dir;
  presets_dir.push("OpenTabletDriver/Presets/");

  if let Ok(presets) = presets_dir.read_dir() {
    let presets = presets
      .map(|p: Result<DirEntry, std::io::Error>| {
        let p = p.unwrap();

        TabletPreset {
          name: p
            .path()
            .clone()
            .file_stem()
            .unwrap()
            .to_string_lossy()
            .into(),
          path: p.path().clone(),
        }
      })
      .collect();

    return Some(presets);
  }

  None
}
