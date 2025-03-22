use slint::{
  Image,
  Model,
  ModelNotify,
};


use crate::utils::matches;
use crate::ui::AppEntry;
use crate::services::{
  Services,
  apps::{
    DesktopEntry,
    DesktopEntriesArc
  }
};



pub struct AppEntries {
  entries: DesktopEntriesArc,
  services: Services<'static>,
  notify: ModelNotify
}

impl AppEntries {
  pub fn new(services: Services<'static>) -> Self {
    Self {
      entries: services.apps().app_list(),
      notify: ModelNotify::default(),
      services,
    }
  }

  pub fn filter_entries(&self, query: &str) {
    let mut entries = self.entries.write().unwrap();

    for entry in entries.iter_mut() {
      if query.trim() == "" && entry.fade {
        entry.fade = false;
      } else if !matches(entry.name.as_str(), &query) {
        entry.fade = true;
      } else {
        entry.fade = false;
      }
    }

    self.notify.reset();
  }
}

impl Model for AppEntries {
  type Data = AppEntry;

  fn row_count(&self) -> usize {
    self.entries.read().unwrap().len()
  }

  fn row_data(&self, row: usize) -> Option<Self::Data> {
    let entries = self.entries.read().unwrap();

    match entries.get(row) {
      Some(r) => Some(r.into()),
      None => None
    }
  }

  fn model_tracker(&self) -> &dyn slint::ModelTracker {
    &self.notify
  }

  fn as_any(&self) -> &(dyn core::any::Any + 'static) {
    self
  }
}


impl Eq for AppEntry {}

impl PartialOrd for AppEntry {
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

impl Ord for AppEntry {
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


impl From<&DesktopEntry> for AppEntry {
  fn from(entry: &DesktopEntry) -> Self {
    let icon = match entry.no_icon {
      true => Image::default(),
      false => Image::from_rgba8_premultiplied(
        entry.cached_icn.clone().into())
    };

    Self {
      name:        entry.name.clone().into(),
      description: entry.description.clone().into(),
      wm_class:    entry.wm_class.clone().into(),
      no_icon:     entry.no_icon,
      fade:        false,
      icon,
    }
  }
}

impl From<DesktopEntry> for AppEntry {
  fn from(entry: DesktopEntry) -> Self {
    let icon = match entry.no_icon {
      true => Image::default(),
      false => Image::from_rgba8_premultiplied(
        entry.cached_icn.into())
    };

    Self {
      name:        entry.name.into(),
      description: entry.description.into(),
      wm_class:    entry.wm_class.into(),
      no_icon:     entry.no_icon,
      fade:        false,
      icon
    }
  }
}

