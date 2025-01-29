use std::error::Error;
use std::cell::RefCell;
use std::rc::Rc;
use std::sync::{Arc, RwLock, Mutex};
use std::path::{self, PathBuf};
use std::collections::HashMap;

use slint::{
  SharedString,
  SharedPixelBuffer,
  Rgba8Pixel,
  Image
};

use crate::utils::matches;
pub use zaemon::apps::DesktopEntry;
use zaemon::apps::get_apps;

use crate::ui::AppEntry;



pub type DesktopEntries = Vec<DesktopEntry>;
pub type SharedAppEntries = Vec<SharedAppEntry>;

pub type DesktopEntriesRc = Rc<RefCell<DesktopEntries>>;
pub type DesktopEntriesArc = Arc<RwLock<DesktopEntries>>;
pub type SharedAppEntriesArc = Arc<Mutex<SharedAppEntries>>;


#[zbus::proxy(
  interface = "org.hypr.Hyprmaster.Apps",
  default_service = "org.hypr.Hyprmaster",
  default_path = "/apps",
  gen_async = true
)]
pub trait Apps {
  async fn all_apps(&self) -> zbus::Result<DesktopEntries>;
}


pub struct AppService<'a> {
  app_list: DesktopEntriesArc,
  app_map: Arc<RwLock<HashMap<String, DesktopEntry>>>,
  proxy: Option<AppsProxy<'a>>
}

impl<'a> AppService<'a> {
  pub async fn new (conn: &zbus::Connection) -> Self {
    Self {
      proxy: AppsProxy::new(conn).await.ok(),
      app_map: Arc::new(RwLock::new(HashMap::new())),
      app_list: Arc::new(RwLock::new(vec![]))
    }
  }

  pub async fn init(&self) ->
    Result<(), Box<dyn Error>>
  {
    let all_apps = match &self.proxy {
      Some(p) => p.all_apps().await?,
      None => get_apps().await
    };

    let mut entries = self.app_list.write().unwrap();

    *entries = all_apps
      .iter()
      .filter(|e| !e.terminal && !e.no_display)
      .cloned()
      .collect();

    let entries = self.app_list.clone();
    let app_map = self.app_map.clone();

    tokio::spawn(async move {
      let entries = entries.read().unwrap();
      let mut app_map = app_map.write().unwrap();

      for entry in entries.iter() {
        app_map.insert(
          entry.wm_class.clone(),
          entry.clone());
      }
    });

    Ok(())
  }

  pub fn app_list(&self) -> DesktopEntriesArc {
    self.app_list.clone()
  }

  pub fn app_map(&self) ->
    Arc<RwLock<HashMap<String, DesktopEntry>>>
  {
    self.app_map.clone()
  }

  pub fn filter_entries(&self, query: String) {
    let entries = self.app_list.clone();

    _ = tokio::spawn(async move {
      let mut entries = entries.write().unwrap();

      for entry in entries.iter_mut() {
        if query.trim() == "" && entry.fade {
          entry.fade = false;
        } else if !matches(entry.name.as_str(), &query) {
          entry.fade = true;
        } else {
          entry.fade = false;
        }
      }
    });
  }
}


pub struct SharedAppEntry {
  pub name:        SharedString,
  pub description: SharedString,
  pub wm_class:    SharedString,
  pub icon:        SharedPixelBuffer<Rgba8Pixel>,
  pub fade:        bool,
}


impl PartialEq for SharedAppEntry {
  fn eq(&self, other: &Self) -> bool {
    self.name == other.name &&
    self.description == other.description &&
    self.wm_class == other.wm_class &&
    self.fade == other.fade
  }
}

impl Eq for SharedAppEntry { }

impl PartialOrd for SharedAppEntry {
  fn partial_cmp(&self, other: &Self) ->
    Option<std::cmp::Ordering>
  {
    if self.fade == other.fade {
      return self.name.partial_cmp(&other.name)
    }

    match self.fade {
      true  => Some(std::cmp::Ordering::Greater),
      false => Some(std::cmp::Ordering::Less)
    }
  }
}

impl Ord for SharedAppEntry {
  fn cmp (&self, other: &Self) -> std::cmp::Ordering {
    if self.fade == other.fade {
      return self.name.cmp(&other.name)
    }

    match self.fade {
      true  => std::cmp::Ordering::Greater,
      false => std::cmp::Ordering::Less
    }
  }
}


impl From<SharedAppEntry> for AppEntry {
  fn from(entry: SharedAppEntry) -> Self {
    Self {
      name: entry.name,
      description: entry.description,
      wm_class: entry.wm_class,
      icon: Image::from_rgba8_premultiplied(entry.icon),
      fade: entry.fade,
      no_icon: false
    }
  }
}

impl From<&SharedAppEntry> for AppEntry {
  fn from(entry: &SharedAppEntry) -> Self {
    Self {
      name        :entry.name.clone(),
      description :entry.description.clone(),
      wm_class    :entry.wm_class.clone(),
      fade        :entry.fade.clone(),
      icon        :Image::from_rgba8_premultiplied(
                    entry.icon.clone()),
      no_icon: false
    }
  }
}

impl From<DesktopEntry> for SharedAppEntry {
  fn from(entry: DesktopEntry) -> Self {
    let fallback =  path::absolute(
        "ui/icons/3d-square.svg")
        .unwrap();

    let icn_path = PathBuf::from(entry.icon_path);
    let icn_path = match icn_path.try_exists().unwrap() {
      true => icn_path,
      false => fallback.clone()
    };

    let icn_path = icn_path.as_path();

    let img = match Image::load_from_path(icn_path) {
      Ok(v) => v.to_rgba8_premultiplied(),
      Err(err) => {
        println!("Couldn't load image from path: {}", err);
        Image::load_from_path(fallback.as_path())
          .expect("Fallback icon should exist")
          .to_rgba8_premultiplied()
      }
    };

    Self {
      name: entry.name.into(),
      description: entry.description.into(),
      wm_class: entry.wm_class.into(),
      fade: false,
      icon: img
        .expect("DesktopEntry.icn_path should exist."),
    }
  }
}

