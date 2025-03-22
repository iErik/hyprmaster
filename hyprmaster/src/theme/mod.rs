use std::collections::HashSet;
use std::path::PathBuf;

use ini::configparser::ini::Ini;

struct GTK2Theme {
  theme_name: String,
  icon_theme_name: String,

  theme_list: String,
}

struct GTK3Theme {
  theme_name: String,
  icon_theme_name: String,

  theme_list: String,
}

struct GTK4Theme {
  theme_name: String,
  icon_theme_name: String,

  theme_list: String,
}

pub struct GTKTheme {
  theme_name: String,
  icon_theme_name: String,

  theme_list: String,
}

pub struct KDETheme {
  theme_name: String,
  icon_theme_name: String,

  theme_list: String,
}

pub struct ThemeManager {
  theme_name: String,
  gtk2_theme: String,
  gtk3_theme: String,
  gtk4_theme: String,
  qt_theme: String,
  cursor_theme: String,
  icon_theme_name: String,
}

fn search_local_themes() {
  let home_dir = std::env::var("HOME").unwrap();
  let xdg_data_home = std::env::var("XDG_DATA_HOME")
    .unwrap_or(format!("{home_dir}/.local/share"));

  let search_dirs: HashSet<String> = HashSet::from([
    // GTK
    "/usr/share/themes".to_string(),
    format!("{home_dir}/.themes"),
    format!("{xdg_data_home}/themes"),
    // KDE/Qt
    format!("{xdg_data_home}/aurorae/themes"),
    "/usr/share/aurorae/themes".to_string(),
  ]);

  for dir in search_dirs {}
}

// Implement cache
// /usr/share/themes
// ~/.themes/
// XDG_DATA_HOME/themes
// ~/.local/share/themes
