use std::sync::{Arc, Mutex};
use std::path::Path;
use std::thread;

use tokio::sync::mpsc::unbounded_channel;
use slint::{
  Image,
  Model,
  ModelNotify,
  Rgba8Pixel,
  SharedPixelBuffer,
  SharedString
};


use crate::utils::matches;
use crate::ui::AppEntry;

use zaemon::DesktopEntry;



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


// TODO: Load fallback in case loading icon_path fails
impl From<DesktopEntry> for SharedAppEntry {
  fn from(entry: DesktopEntry) -> Self {
    let img = match Image::load_from_path(
      Path::new(entry.icon_path.as_str())
    ) {
      Ok(v) => v.to_rgba8_premultiplied(),
      Err(err) => {
        println!("Couldn't load image from path: {}", err);
        None
      }
    };

    Self {
      name: entry.name.into(),
      description: entry.description.into(),
      wm_class: entry.wm_class.into(),
      icon: img.unwrap(),
      fade: false,
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
    }
  }
}



pub(super) struct AppEntries {
  pub entries: Arc<Mutex<Vec<SharedAppEntry>>>,
  notify: ModelNotify,
}

impl AppEntries {
  pub fn new() -> Self {
    Self {
      entries: Arc::new(Mutex::new(vec![])),
      notify: ModelNotify::default(),
    }
  }

  pub async fn filter_entries(&self, query: &str) {
    let (ssx, mut rrx) = unbounded_channel();
    let entries = self.entries.clone();
    let q = String::from(query);
    let q = Arc::new(q);

    thread::spawn(move || {
      let q = q.clone();
      let mut elock = entries.lock().unwrap();

      for entry in elock.iter_mut() {

        if q.trim() == "" {
          entry.fade = false
        } else if !matches(entry.name.as_str(), &q) {
          entry.fade = true;
        } else {
          entry.fade = false;
          println!("Matches!: {}", entry.name);
        }
      }

      elock.sort();
      ssx.send(true).expect("Sender is poisoned");
    });

    while let Some(done) = rrx.recv().await {
      if done {
        self.notify.reset();
      }
    }
  }

  pub async fn populate_async(&self) {
    /*
    let (sx, mut rx)   = unbounded_channel();
    let entries_handle = self.entries.clone();

    thread::spawn(move || {
      let apps: Vec<SharedAppEntry> = apps::get_apps()
        .into_iter()
        .filter(|entry| {
          !entry.no_display && !entry.terminal
        })
        .map(|entry| entry.into())
        .collect();

      sx.send(apps).expect("Sender poisoned");
    });

    while let Some(entries) = rx.recv().await {
      let mut lock = entries_handle.lock().unwrap();
      *lock = entries;

      self.notify.row_added(0, lock.len());
    }
    */
  }
}

impl Model for  AppEntries {
  type Data = AppEntry;

  fn row_count(&self) -> usize {
    match self.entries.lock() {
      Ok(v) => v.len(),
      _ => 0
    }
  }

  fn row_data(&self, row: usize) -> Option<Self::Data> {
    let entries = self.entries.lock().unwrap();

    match entries.get(row) {
      Some(data) => Some(AppEntry::from(data)),
      None => None,
    }
  }

  fn model_tracker(&self) -> &dyn slint::ModelTracker {
    &self.notify
  }

  fn as_any(&self) -> &dyn core::any::Any {
    self
  }
}

