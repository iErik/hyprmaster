use std::path::{Path, PathBuf};
use std::collections::HashSet;
use std::cmp::Ordering;
use std::ffi::OsStr;
use std::env;

use zbus::interface;
use tokio::task::JoinSet;
use ini::configparser::ini::Ini;
use walkdir::{WalkDir, DirEntry};

use crate::dconf;



pub struct IconsObject {
  icon_cache: Ini,
  icon_theme: String,
}

impl IconsObject {
  pub fn new() -> Self {
    let mut config = Ini::new();
    let home_dir = env::var("HOME")
      .unwrap_or("~".to_string());

    let config_dir = std::env::var("XDG_CONFIG_HOME")
      .unwrap_or(format!("{home_dir}/.config"));
    let config_dir = PathBuf::from(format!(
      "{config_dir}/hyprmaster/icons.cache"
    ));

    _ = config.load(config_dir.to_str().unwrap());


    Self {
      icon_theme: get_current_theme(),
      icon_cache: config,
    }
  }

  fn cache_get(&self, icn_name: &str) -> Option<String> {
    let path :Option<String> = self.icon_cache
      .get(self.icon_theme.as_str(), icn_name);

    return match Path::new(&path.clone()?).exists() {
      true => path,
      false => None
    }
  }

  fn cache_set(&mut self, key: &str, value: String) {
    self.icon_cache.set(
      self.icon_theme.as_str(),
      key,
      Some(value),
    );
  }
}


// TODO:
// - Cache all icons beferohand
// - Watch for icon theme changes and emit signals
// - Watch for icon themes being added/remove and
//   emit signals
#[interface(name = "org.hypr.Hyprmaster.Icons")]
impl IconsObject {
  pub async fn get_icon(&mut self, icn_name: &str) -> String {
    if let Some(icn) = self.cache_get(icn_name) {
      return icn
    }

    match get_icon(&icn_name.to_string()).await {
      Some(result) => {
        self.cache_set(icn_name, result.clone());
        result
      },
      None => String::from("")
    }
  }
}



fn get_icon_themes() {
  let home_dir = std::env::var("HOME").unwrap();
  let xdg_data_home = std::env::var("XDG_DATA_HOME")
    .unwrap_or(format!("{home_dir}/.local/share"));

  let search_dirs: HashSet<String> = HashSet::from([
    "/usr/share/icons".to_string(),
    format!("{home_dir}/.icons"),
    format!("{xdg_data_home}/icons"),
  ]);
}

pub async fn get_icon(icon_name: &String) -> Option<String> {
  let lookup_dirs = get_lookup_dirs();

  match icon_lookup(icon_name, lookup_dirs).await {
    Some(icn) => Some(icn),
    _ => icon_lookup(icon_name, get_backup_dirs()).await
  }
}

async fn icon_lookup(
  icon_name: &String,
  dirs: Vec<PathBuf>,
) -> Option<String> {
  let mut lookup_set = JoinSet::new();

  let sort_fn = |a: &DirEntry, b: &DirEntry| -> Ordering {
    let ext_a = a.path().extension()
      .unwrap_or(OsStr::new(""));
    let ext_b = b.path().extension()
      .unwrap_or(OsStr::new(""));

    if ext_a == ext_b {
      return a.file_name().cmp(b.file_name())
    }

    if ext_a == "svg" { return Ordering::Less }
    if ext_b == "svg" { return Ordering::Greater }

    a.file_name().cmp(b.file_name())
  };


  for dir in dirs {
    let walker = WalkDir::new(dir)
      .contents_first(true)
      .sort_by(sort_fn)
      .follow_links(true);

    for entry in walker {
      let path: PathBuf = match entry {
        Ok(v) => v.into_path(),
        _ => continue
      };

      let icon = icon_name.clone();

      lookup_set.spawn_blocking(move || {
        let stem = path.file_stem()?
          .to_string_lossy().to_string();

        match stem == icon {
          true => Some(path.to_string_lossy().to_string()),
          _ => None
        }
      });
    }
  }

  while let Some(res) = lookup_set.join_next().await {
    if res.is_err() { continue }

    if let Some(icon) = res.unwrap() {
      lookup_set.abort_all();
      return Some(icon)
    }
  }

  None
}

pub fn get_icon_sync(icon_name: &String) -> Option<String> {
  let lookup_dirs = get_lookup_dirs();

  match icon_lookup_sync(icon_name, lookup_dirs) {
    Some(icn) => Some(icn),
    _ => icon_lookup_sync(icon_name, get_backup_dirs())
  }
}

fn icon_lookup_sync(
  icon_name: &String,
  dirs: Vec<PathBuf>
) -> Option<String> {
  let sort_fn = |a: &DirEntry, b: &DirEntry| -> Ordering {
    let ext_a = a.path().extension()
      .unwrap_or(OsStr::new(""));
    let ext_b = b.path().extension()
      .unwrap_or(OsStr::new(""));

    if ext_a == ext_b {
      return a.file_name().cmp(b.file_name())
    }

    if ext_a == "svg" { return Ordering::Less }
    if ext_b == "svg" { return Ordering::Greater }

    a.file_name().cmp(b.file_name())
  };


  for dir in dirs {
    let walker = WalkDir::new(dir)
      .contents_first(true)
      .sort_by(sort_fn)
      .follow_links(false);

    for entry in walker {
      let path: PathBuf = match entry {
        Ok(v) => v.into_path(),
        _ => continue
      };

      let stem = path.file_stem()?
        .to_string_lossy().to_string();
      println!("checking: {}", stem);

      if stem == *icon_name {
        return Some(path.to_string_lossy().to_string())
      }
    }
  }

  None
}

// TODO: Check for supported icon types by looking
// at the file extension
fn file_matches_icon(
  path: impl Into<PathBuf>,
  icon: impl Into<String>
) -> bool {
  let path: PathBuf = path.into();
  let icon: String = icon.into();

  if path.exists() && path.is_file() {
    let stem = path.file_stem().unwrap();
    let stem_str: String = stem.to_string_lossy().into();

    return stem_str == icon;
  }

  false
}

fn get_current_theme() -> String {
  match dconf::interface("icon-theme") {
    Ok(v) => v,
    _ => String::from("hicolor")
  }
}

fn get_backup_dirs() -> Vec<PathBuf> {
  let xdg_home = env::var("HOME").unwrap();
  let home_dir = Path::new(&xdg_home);

  let dirs: [PathBuf; 4] = [
    home_dir.join(".local/share/icons/hicolor"),
    home_dir.join(".local/share/icons/pixmaps"),
    PathBuf::from("/usr/share/icons/hicolor"),
    PathBuf::from("/usr/share/pixmaps")
  ];

  dirs.into_iter()
    .filter(|p| p.exists())
    .collect()
}

fn get_lookup_dirs() -> Vec<PathBuf> {
  let xdg_home = env::var("HOME").unwrap();
  let icon_theme = get_current_theme();

  vec![
    PathBuf::from(
      format!("{xdg_home}/.local/share/{icon_theme}")
    ),
    PathBuf::from(format!("/usr/share/icons/{icon_theme}"))
  ].into_iter().filter(|p| p.exists()).collect()
}
