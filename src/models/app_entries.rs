use std::sync::{Arc, Mutex};
use std::thread;

use slint::{Model, ModelNotify, Rgba8Pixel};
use tokio::sync::mpsc::unbounded_channel;

use crate::ui::{self, AppEntry};
use crate::apps::{self, DesktopEntry};

pub struct SharedAppEntry {
  pub name: slint::SharedString,
  pub description: slint::SharedString,
  pub wm_class: slint::SharedString,
  pub icon: slint::SharedPixelBuffer<Rgba8Pixel>,
  pub fade: bool,
}

impl From<SharedAppEntry> for AppEntry {
  fn from(entry: SharedAppEntry) -> Self {
    Self {
      name: entry.name,
      description: entry.description,
      wm_class: entry.wm_class,
      icon: slint::Image::from_rgba8_premultiplied(
        entry.icon,
      ),
      fade: entry.fade,
    }
  }
}

impl From<&SharedAppEntry> for AppEntry {
  fn from(entry: &SharedAppEntry) -> Self {
    Self {
      name: entry.name.clone(),
      description: entry.description.clone(),
      wm_class: entry.wm_class.clone(),
      icon: slint::Image::from_rgba8_premultiplied(
        entry.icon.clone(),
      ),
      fade: entry.fade.clone(),
    }
  }
}

impl From<DesktopEntry> for SharedAppEntry {
  fn from(entry: DesktopEntry) -> Self {
    let img = image::open(entry.icon_path.as_path())
      .expect("Failed to load entry icon")
      .into_rgba8();

    Self {
      name: entry.name.into(),
      description: entry.description.into(),
      wm_class: entry.wm_class.into(),
      icon: slint::SharedPixelBuffer::
        <Rgba8Pixel>::clone_from_slice(
          img.as_raw(),
          img.width(),
          img.height()),
      fade: false,
    }
  }
}

struct SharedModel {
  pub entries: Arc<Mutex<Vec<SharedAppEntry>>>,
  notify: ModelNotify,
}

impl SharedModel {
  pub fn new() -> Self {
    SharedModel {
      entries: Arc::new(Mutex::new(vec![])),
      notify: ModelNotify::default(),
    }
  }

  pub fn filter(&mut self, query: &str) {
    let test = self.entries.clone();
    //let test = test.get_mut().unwrap();
    let mut test = test.lock().unwrap();
    let query_vec: Vec<&str> = String::from(query.clone())
      .split(' ')
      .collect();

    let matches = move |name: slint::SharedString| {
      let name_vec: Vec<&str> = String::from(name.clone())
        .split(' ')
        .collect();
    };

    test.iter_mut().map(|entry| {
      let matches: bool;
      let name: Vec<&str> =
        String::from(entry.name.clone())
          .split(' ')
          .collect();

      entry
    });

    thread::spawn(move || {});
  }

  pub async fn populate_async(&self) {
    let (sx, mut rx) = unbounded_channel();
    let entries_handle = self.entries.clone();

    thread::spawn(move || {
      let apps: Vec<DesktopEntry> = apps::get_apps()
        .into_iter()
        .filter(|entry| {
          !entry.no_display && !entry.terminal
        })
        .collect();

      sx.send(apps).expect("Sender poisoned");
    });

    while let Some(entries) = rx.recv().await {
      let mut lock = entries_handle.lock().unwrap();
      *lock = entries
        .into_iter()
        .map(|entry| entry.into())
        .collect();

      self.notify.row_added(0, lock.len());
    }
  }
}

impl Model for SharedModel {
  type Data = AppEntry;

  fn row_count(&self) -> usize {
    let entries = self.entries.lock().expect(concat!(
      "SharedModel.row_count: ",
      "entries lock is poisoned"
    ));

    entries.len()
  }

  fn row_data(&self, row: usize) -> Option<Self::Data> {
    let entries = self.entries.lock().expect(concat!(
      "SharedModel.row_data:",
      "entries lock is poisoned"
    ));

    let row = match entries.get(row) {
      Some(data) => Some(AppEntry::from(data)),
      None => None,
    };

    row
  }

  fn model_tracker(&self) -> &dyn slint::ModelTracker {
    &self.notify
  }

  fn as_any(&self) -> &dyn core::any::Any {
    self
  }
}

pub struct AppEntriesModel {
  pub entries: Arc<Mutex<Vec<AppEntry>>>,
  notify: ModelNotify,
}

impl AppEntriesModel {
  pub fn new() -> Self {
    AppEntriesModel {
      entries: Arc::new(Mutex::new(vec![])),
      notify: ModelNotify::default(),
    }
  }

  // TODO: get_apps optimization using cache
  pub fn populate(&self) {
    let apps: Vec<AppEntry> = apps::get_apps()
      .into_iter()
      .filter(|entry| !entry.no_display && !entry.terminal)
      .map(|entry| entry.into())
      .collect();

    //let mut entries = self.entries.write().unwrap();
    let mut entries = self.entries.lock().unwrap();
    *entries = apps;
    self.notify.row_added(0, entries.len());
  }

  pub async fn populate_async(&self) {
    let (sx, mut rx) = unbounded_channel();
    let entries_handle = self.entries.clone();

    thread::spawn(move || {
      let apps: Vec<DesktopEntry> = apps::get_apps()
        .into_iter()
        .filter(|entry| {
          !entry.no_display && !entry.terminal
        })
        .collect();

      sx.send(apps).expect("Sender poisoned");
    });

    while let Some(entries) = rx.recv().await {
      //let mut lock = entries_handle.write().unwrap();
      let mut lock = entries_handle.lock().unwrap();
      *lock = entries
        .into_iter()
        .map(|entry| entry.into())
        .collect();

      self.notify.row_added(0, lock.len());
    }
  }

  pub fn filter(&self, query: &str) {
    let b = &self.entries.clone();
  }
}

impl Model for AppEntriesModel {
  type Data = AppEntry;

  fn row_count(&self) -> usize {
    let entries = self.entries.read().expect(concat!(
      "AppEntriesModel.row_count: ",
      "app_entries lock is poisoned"
    ));

    entries.len()
  }

  fn row_data(&self, row: usize) -> Option<Self::Data> {
    let entries = self.entries.read().expect(concat!(
      "AppEntriesModel.row_data: ",
      "app_entries lock is poisoned"
    ));

    entries.get(row).cloned()
  }

  fn model_tracker(&self) -> &dyn slint::ModelTracker {
    &self.notify
  }

  //fn set_row_data(&self, row: usize, data: Self::Data) {}

  fn as_any(&self) -> &dyn core::any::Any {
    self
  }
}

struct AppEntriesCtrl {
  window: ui::MainWindow,
}

impl AppEntriesCtrl {}
