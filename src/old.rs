use std::collections::HashSet;
use std::path::{self, PathBuf};
use std::rc::Rc;
use std::sync::{Arc, Mutex};
use std::thread;

use freedesktop_entry_parser::parse_entry;
use walkdir::WalkDir;

use slint::{Image, ModelRc, VecModel};

pub mod ui {
  slint::include_modules!();
}

use ui::*;

#[derive(Default, Clone, PartialEq, Eq)]
struct PreAppEntry {
  name: String,
  wm_class: String,
  icon_path: PathBuf,
  icon_name: String,
  description: String,
  app_path_bin: String,
  app_desktop_path: String,
}

impl PartialOrd for PreAppEntry {
  fn partial_cmp(
    &self,
    other: &Self,
  ) -> Option<std::cmp::Ordering> {
    self.name.partial_cmp(&other.name)
  }
}

impl Ord for PreAppEntry {
  fn cmp(&self, other: &Self) -> std::cmp::Ordering {
    self.name.cmp(&other.name)
  }
}

impl Into<AppEntry> for PreAppEntry {
  fn into(self) -> AppEntry {
    AppEntry {
      fade: false,
      name: self.name.into(),
      description: self.description.into(),
      wm_class: self.wm_class.into(),
      icon: Image::load_from_path(self.icon_path.as_path())
        .unwrap(),
    }
  }
}

fn parse_desktop_file(
  desktop_file_path: PathBuf,
) -> Option<PreAppEntry> {
  let mut app = PreAppEntry::default();
  app.app_desktop_path = desktop_file_path
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

  if entry.has_attr("NoDisplay")
    && entry.attr("NoDisplay").unwrap() == "true"
  {
    return None;
  }

  if entry.has_attr("Name") {
    app.name = entry.attr("Name").unwrap().to_string();
  } else {
    return None;
  }

  if entry.has_attr("Comment") {
    app.description = match entry.attr("Comment") {
      Some(txt) => txt.to_string(),
      None => String::from(""),
    }
  }

  if entry.has_attr("Exec") {
    app.app_path_bin =
      entry.attr("Exec").unwrap().to_string();
  } else {
    return None;
  }

  if entry.has_attr("Icon") {
    let icon: String =
      entry.attr("Icon").unwrap().to_string();

    let mut mngr = IconManager::new();
    let path = mngr.get_icon(
      &icon
        .clone()
        .replace("\"", "")
        .replace("'", ""),
    );

    app.icon_path = match path {
      Some(path) => path.into(),
      None => {
        path::absolute("ui/icons/3d-square.svg").unwrap()
      }
    };

    app.icon_name = icon;
  } else {
    app.icon_path =
      path::absolute("ui/icons/3d-square.svg").unwrap();
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

fn get_apps() -> Vec<PreAppEntry> {
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

  let apps: Vec<PreAppEntry> = Vec::new();
  let apps = Arc::new(Mutex::new(apps));
  let mut handles = vec![];

  for dir in search_dirs {
    let dir = PathBuf::from(dir);
    if !dir.exists() {
      continue;
    }

    let apps_handle = Arc::clone(&apps);

    let join_handle = thread::spawn(move || {
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
          let app = parse_desktop_file(path.to_path_buf());

          match app {
            Some(app) => {
              let mut lock = apps_handle.lock().unwrap();
              lock.push(app);
            }
            None => {
              continue;
            }
          }
        }
      }
    });

    handles.push(join_handle);
  }

  for handle in handles {
    _ = handle.join();
  }

  let mut apps = Arc::clone(&apps)
    .lock()
    .unwrap()
    .to_vec();

  apps.sort();

  apps
}

fn old_man() -> Result<(), slint::PlatformError> {
  let main_window = Arc::new(MainWindow::new().unwrap());
  let handle = main_window.as_weak();

  thread::spawn(move || {
    let raw_entries = get_apps();

    let raw_entries = Arc::new(Mutex::new(raw_entries));
    let handle = handle.clone();

    slint::invoke_from_event_loop(move || {
      let l: Vec<AppEntry> = raw_entries
        .lock()
        .unwrap()
        .clone()
        .iter()
        .map(|entry| entry.clone().into())
        .collect();

      println!("Attaching app-entries collection");

      let model = ModelRc::from(Rc::new(VecModel::from(l)));

      handle
        .upgrade()
        .unwrap()
        .global::<TabletUIState>()
        .set_app_entries(model);
    })
  });

  main_window.run()
}
