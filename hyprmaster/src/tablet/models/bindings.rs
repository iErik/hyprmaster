use slint::{
  Model,
  ModelNotify
};


use crate::ui::AppBinding;
use crate::services::{
  Services,
  tablet::BindingsRc
};


pub struct AppBindings {
  bindings: BindingsRc,
  services: Services<'static>,
  notify: ModelNotify,
}

impl AppBindings {
  pub fn new(services: Services<'static>) -> Self {
    Self {
      bindings: services.tablet().bindings(),
      notify: ModelNotify::default(),
      services,
    }
  }

  fn bindings (&self) -> Option<Vec<AppBinding>> {
    let apps = self.services.apps().app_map();
    let apps = match apps.try_read() {
      Ok(v) => v,
      Err(_) => return None
    };

    let list = self.bindings.borrow();

    Some(list
      .iter()
      .filter(|(k, _)| apps.contains_key(k.as_str()))
      .map(|(k, v)| AppBinding {
        preset: v.clone().into(),
        app: apps.get(k).unwrap().into()
      })
    .collect())
  }
}


impl Model for AppBindings {
  type Data = AppBinding;

  fn row_count(&self) -> usize {
    match self.bindings() {
      Some(v) => v.len(),
      None => 0
    }
  }

  fn row_data(&self, row: usize) -> Option<Self::Data>  {
    match self.bindings() {
      Some(v) => v.get(row).cloned(),
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
