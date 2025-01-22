use std::collections::HashMap;
use std::fs;
use std::env::var;
use tokio::net::UnixStream;

use zbus::interface;

use crate::dconf;


type TabletBindings = HashMap<String, Vec<String>>;

pub(crate) struct TabletInterface {
  presets: Vec<String>,
  app_bindings: TabletBindings
}

// Save tablet settings into gsettings
#[interface(name = "org.hypr.Hyprmaster.Tablet")]
impl TabletInterface {
  #[zbus(property)]
  fn presets(&self) -> Vec<String> {
    self.presets.clone()
  }

  #[zbus(property)]
  fn app_bindings(&self) -> TabletBindings {
    self.app_bindings.clone()
  }

  fn add_preset(&self) {}

  fn remove_preset(&self){}

  //fn current_preset(&self) { }
}

impl TabletInterface {
  pub fn new() -> Self {
    Self {
      presets: get_presets(),
      app_bindings: get_bindings()
    }
  }

  pub async fn listen(conn: &zbus::Connection) ->
    Result<(), Box<dyn std::error::Error>>
  {
    let rtm_dir = var("XDG_RUNTINE_DIR")?;
    let his_dir = var("HYPRLAND_INSTANCE_SIGNATURE")?;
    let sock_src = format!(
      "{rtm_dir}/hypr/{his_dir}/.socket2.sock");

    let sock = UnixStream::connect(sock_src).await?;

    Ok(())
  }
}

fn get_presets() -> Vec<String> {
  let base_dir    = "OpenTabletDriver/Presets";
  let home_dir    = var("HOME").unwrap();
  let presets_dir = var("XDG_CONFIG_HOME")
    .unwrap_or(format!("{home_dir}/.config/{base_dir}"));

  let paths = match fs::read_dir(presets_dir) {
    Ok(dirs) => dirs,
    Err(_) => return vec![]
  };

  paths
    .map(|d| d.unwrap()
      .file_name()
      .to_string_lossy()
      .to_string())
    .collect()
}


fn parse_bindings(val: String) -> TabletBindings {
  let mut bindings_map = HashMap::new();

  val
    .trim()
    .strip_suffix(['}', ']']).unwrap()
    .strip_prefix(['{', '[']).unwrap()
    .split(",")
    .for_each(|s| {
      let s: Vec<&str> = s.split(':').collect();

      let key = s.first().unwrap().trim();
      let list: Vec<String> = s.last().unwrap()
        .trim()
        .strip_suffix(['}', ']']).unwrap()
        .strip_prefix(['{', '[']).unwrap()
        .split(",")
        .map(|s| s.to_string())
        .collect();

      bindings_map.insert(key.to_string(), list);
    });

  bindings_map
}

fn get_bindings() -> TabletBindings {
  let bindings =
    dconf::get("org.hypr.Hyprmaster.tablet", "bindings")
    .unwrap();

  parse_bindings(bindings)
}
