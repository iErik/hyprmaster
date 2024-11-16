use std::path::PathBuf;
use std::fs::FileType;
use std::collections::HashSet;
use std::process::Command;
use std::ffi::OsStr;

use std::cmp;

use walkdir::{WalkDir, DirEntry};
use ini::configparser::ini::Ini;

pub struct IconManager {
  cache_src: PathBuf,
  cache_map: Ini,
  current_icon_theme: String,

  lookup_dirs: Vec<PathBuf>,
  backup_dirs: Vec<PathBuf>,
}

impl IconManager {
  pub fn new() -> Self {
    let mut config = Ini::new();
    let home_dir =
      std::env::var("HOME").unwrap_or("~".to_string());
    let xdg_data_home = std::env::var("XDG_DATA_HOME")
      .unwrap_or(format!("{home_dir}/.local/share"));

    let config_dir = std::env::var("XDG_CONFIG_HOME")
      .unwrap_or(format!("{home_dir}/.config"));
    let config_dir = PathBuf::from(format!(
      "{config_dir}/hypr/icons.cache"
    ));

    _ = config.load(config_dir.to_str().unwrap());

    let icon_pack = get_current_icon_theme();

    Self {
      cache_map: config,
      cache_src: config_dir,
      current_icon_theme: icon_pack.clone(),
      lookup_dirs: vec![
        PathBuf::from(format!(
          "{xdg_data_home}/{icon_pack}"
        )),
        PathBuf::from(format!(
          "/usr/share/icons/{icon_pack}"
        )),
      ],
      backup_dirs: get_backup_dirs(),
    }
  }

  pub fn cache_get(
    &self,
    icn_name: &str,
  ) -> Option<String> {
    self
      .cache_map
      .get(self.current_icon_theme.as_str(), icn_name)
  }

  pub fn cache_set(
    &mut self,
    key: &str,
    value: Option<String>,
  ) -> Option<Option<String>> {
    self.cache_map.set(
      self.current_icon_theme.as_str(),
      key,
      value,
    )
  }

  pub fn cache_write(
    &mut self,
  ) -> Result<(), std::io::Error> {
    self
      .cache_map
      .write(self.cache_src.to_str().unwrap())
  }

  pub fn get_svg_icon(
    &mut self,
    icon_name: &String,
  ) -> Option<String> {
    if let Some(icn) = self.cache_get(&icon_name) {
      return Some(icn);
    }

    let filter_fn = |entry: &DirEntry| -> bool {
      let file_type: FileType = entry.file_type();

      if file_type.is_file() {
        return match entry.path().extension() {
          Some(ext) => ext == "svg",
          None => false,
        };
      }

      true
    };

    for dir in &self.lookup_dirs {
      if !dir.exists() {
        continue;
      }

      let dir_walker = WalkDir::new(dir)
        .follow_links(true)
        .into_iter()
        .filter_entry(filter_fn);

      for entry in dir_walker {
        if entry.is_err() {
          continue;
        }

        let entry = entry.unwrap();
        let path = entry.path();

        if file_matches_icon(path, icon_name) {
          let icn_path =
            Some(path.to_string_lossy().into());
          self.cache_set(icon_name, icn_path.clone());
          _ = self.cache_write();
          return icn_path;
        }
      }
    }

    None
  }

  pub fn search_icon_themes() {
    let home_dir = std::env::var("HOME").unwrap();
    let xdg_data_home = std::env::var("XDG_DATA_HOME")
      .unwrap_or(format!("{home_dir}/.local/share"));

    let search_dirs: HashSet<String> = HashSet::from([
      "/usr/share/icons".to_string(),
      format!("{home_dir}/.icons"),
      format!("{xdg_data_home}/icons"),
    ]);

    for dir in search_dirs {}
  }

  pub fn get_icon(
    &mut self,
    icon_name: &String,
  ) -> Option<String> {
    if PathBuf::from(icon_name).exists() {
      return match icon_name.ends_with(".ico") {
        true => None,
        false => Some(icon_name.to_string()),
      };
    }

    if let Some(icn) = self.cache_get(&icon_name) {
      return Some(icn);
    }

    let sort_fn =
      |a: &DirEntry, b: &DirEntry| -> cmp::Ordering {
        let ext_a = match a.path().extension() {
          Some(ext) => ext,
          None => OsStr::new(""),
        };

        let ext_b = match a.path().extension() {
          Some(ext) => ext,
          None => OsStr::new(""),
        };

        if ext_a == ext_b {
          return a.file_name().cmp(b.file_name());
        }

        if ext_a == "svg" {
          return cmp::Ordering::Less;
        }

        if ext_b == "svg" {
          return cmp::Ordering::Greater;
        }

        a.file_name().cmp(b.file_name())
      };

    for dir in &self.lookup_dirs {
      if !dir.exists() {
        continue;
      }

      let dir_walker = WalkDir::new(dir)
        .follow_links(true)
        .sort_by(sort_fn);

      for entry in dir_walker {
        if entry.is_err() {
          continue;
        }

        let entry = entry.unwrap();
        let path = entry.path();

        if file_matches_icon(path, icon_name) {
          let icn_path =
            Some(path.to_string_lossy().into());
          self.cache_set(icon_name, icn_path.clone());
          _ = self.cache_write();
          return icn_path;
        }
      }
    }

    for dir in &self.backup_dirs {
      if !dir.exists() {
        continue;
      }

      let dir_walker = WalkDir::new(dir).follow_links(true);

      for entry in dir_walker {
        if entry.is_err() {
          continue;
        }

        let entry = entry.unwrap();
        let path = entry.path();

        if file_matches_icon(path, icon_name) {
          let icn_path =
            Some(path.to_string_lossy().into());
          self.cache_set(icon_name, icn_path.clone());
          _ = self.cache_write();
          return icn_path;
        }
      }
    }

    None
  }
}

pub fn file_matches_icon(
  path: impl Into<PathBuf>,
  icon: impl Into<String>,
) -> bool {
  let path: PathBuf = path.into();
  let icon = icon.into();

  if path.exists() && path.is_file() {
    let stem = path.file_stem().unwrap();
    let stem_str: String = stem.to_string_lossy().into();

    return stem_str == icon;
  }

  false
}

pub fn get_backup_dirs() -> Vec<PathBuf> {
  let home_dir =
    PathBuf::from(std::env::var("HOME").unwrap());

  let mut local_hicolor = home_dir.clone();
  local_hicolor.push(".local/share/icons/hicolor");

  let mut local_pixmap = home_dir.clone();
  local_pixmap.push(".local/share/icons/pixmaps");

  let usr_hicolor =
    PathBuf::from("/usr/share/icons/hicolor");
  let usr_pixmap = PathBuf::from("/usr/share/pixmaps");

  let mut dirs = Vec::<PathBuf>::new();

  if local_hicolor.exists() {
    dirs.push(local_hicolor);
  }

  if local_pixmap.exists() {
    dirs.push(local_pixmap);
  }

  if usr_hicolor.exists() {
    dirs.push(usr_hicolor);
  }

  if usr_pixmap.exists() {
    dirs.push(usr_pixmap);
  }

  dirs
}

pub fn get_current_icon_theme() -> String {
  let cmd = Command::new("gsettings")
    .arg("get")
    .arg("org.gnome.desktop.interface")
    .arg("icon-theme")
    .output()
    .expect("Error getting current icon theme.");

  let icon_pack = String::from_utf8(cmd.stdout)
    .unwrap_or("hicolor".to_string())
    .replace("'", "")
    .replace("\n", "");

  icon_pack
}
