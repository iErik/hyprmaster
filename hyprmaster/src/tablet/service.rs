use std::collections::HashSet;
use std::path::{self, PathBuf};
use std::sync::{Arc, Mutex};
use std::thread;

use walkdir::WalkDir;
use freedesktop_entry_parser::parse_entry;
use zaemon::apps::DesktopEntry;

use crate::icons::IconManager;



pub fn parse_app_entry(
  desktop_file_path: PathBuf,
) -> Option<DesktopEntry> {
  let mut app = DesktopEntry::default();
  app.entry_path = desktop_file_path
    .clone()
    .into_os_string()
    .into_string()
    .unwrap();

  let desktop_file_path_str =
    desktop_file_path.to_str().unwrap();
  let entry = match parse_entry(desktop_file_path_str) {
    Ok(entry) => entry,
    Err(_) => return None,
  };

  if !entry.has_section("Desktop Entry") {
    return None;
  }

  let entry = entry.section("Desktop Entry");

  if entry.has_attr("Exec") {
    app.exec = entry.attr("Exec").unwrap().to_string();
  } else {
    return None;
  }

  if entry.has_attr("Name") {
    app.name = entry.attr("Name").unwrap().to_string();
  } else {
    return None;
  }

  if entry.has_attr("NoDisplay") {
    app.no_display = match entry.attr("NoDisplay").unwrap()
    {
      "true" | "True" => true,
      "false" | "False" => false,
      _ => false,
    }
  }

  if entry.has_attr("Terminal") {
    app.terminal = match entry.attr("Terminal").unwrap() {
      "true" | "True" => true,
      "false" | "False" => false,
      _ => false,
    }
  }

  if entry.has_attr("Comment") {
    app.description = match entry.attr("Comment") {
      Some(txt) => txt.to_string(),
      None => String::from(""),
    }
  }

  // TODO: Icon fetching is a major issue here
  if entry.has_attr("Icon") {
    let icon: String = entry
      .attr("Icon")
      .unwrap()
      .to_string()
      .replace('"', "")
      .replace('\'', "");

    let path = IconManager::new().get_icon(&icon.clone());

    app.icon_path = match path {
      Some(path) => path.into(),
      None => {
        // TODO: Perhaps turn this into a constant
        path::absolute("ui/icons/3d-square.svg")
          .unwrap()
          .to_string_lossy()
          .to_string()
      }
    };

    app.icon_name = icon;
  } else {
    app.icon_path = path::absolute("ui/icons/3d-square.svg")
      .unwrap()
      .to_string_lossy()
      .to_string();
  }

  if entry.has_attr("StartupWMClass") {
    let wm_class = entry.attr("StartupWMClass").unwrap();
    app.wm_class = wm_class.to_string();
  } else {
    let exec: String = entry
      .attr("Exec")
      .unwrap()
      .split(' ')
      .nth(0)
      .unwrap()
      .into();

    let exec: String = match PathBuf::from(&exec).exists() {
      true => PathBuf::from(exec)
        .file_stem()
        .unwrap_or(std::ffi::OsStr::new(""))
        .to_string_lossy()
        .into(),
      false => exec,
    };

    app.wm_class = exec;
  }

  Some(app)
}

pub fn get_apps() -> Vec<DesktopEntry> {
  let xdg_data_dirs: Vec<String> =
    std::env::var("XDG_DATA_DIRS")
      .unwrap_or("/usr/share".to_string())
      .split(':')
      .map(|s| match s.ends_with("/") {
        true => format!("{s}applications"),
        false => format!("{s}/applications"),
      })
      .collect();

  // Ensure we don't have duplicate paths
  let mut search_dirs: HashSet<String> =
    xdg_data_dirs.iter().cloned().collect();

  search_dirs.insert("/usr/share/applications".to_string());

  // get home dir of current user
  let home_dir = std::env::var("HOME").unwrap();
  let home_path = PathBuf::from(home_dir);
  let local_share_apps = home_path
    .join(".local/share/applications")
    .into_os_string()
    .into_string()
    .unwrap();

  search_dirs.insert(local_share_apps);

  let apps: Vec<DesktopEntry> = Vec::new();
  let apps = Arc::new(Mutex::new(apps));

  thread::scope(|scope| {
    for dir in search_dirs {
      let dir = PathBuf::from(dir);
      if !dir.exists() {
        continue;
      }

      let app_handle = Arc::clone(&apps);

      scope.spawn(move || {
        for entry in WalkDir::new(dir.clone()) {
          if entry.is_err() {
            continue;
          };

          let entry = entry.unwrap();
          let path = entry.path();
          if path.extension().is_none() || path.is_dir() {
            continue;
          }

          if path.extension().unwrap() == "desktop" {
            let app = parse_app_entry(path.to_path_buf());

            match app {
              Some(app) => {
                let mut lock = app_handle.lock().unwrap();
                lock.push(app);
              }
              None => {
                continue;
              }
            }
          }
        }
      });
    }
  });

  let mut apps = Arc::clone(&apps)
    .lock()
    .unwrap()
    .to_vec();

  apps.sort();
  apps
}

