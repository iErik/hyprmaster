use slint::{
  Model,
  ModelNotify,
  SharedString
};

use crate::services::{
  Services,
  tablet::PresetsRc
};


pub struct TabletPresets {
  presets: PresetsRc,
  services: Services<'static>,
  notify: ModelNotify
}

impl TabletPresets {
  pub fn new(services: Services<'static>) -> Self {

    Self {
      presets: services.tablet().presets(),
      services,
      notify: ModelNotify::default()
    }
  }
}


impl Model for TabletPresets {
  type Data = SharedString;

  fn row_count(&self) -> usize {
    self.presets.borrow().len()
  }

  fn row_data(&self, row: usize) -> Option<Self::Data> {
    let presets = self.presets.borrow();
    presets.get(row).cloned()
  }

  fn model_tracker(&self) -> &dyn slint::ModelTracker {
    &self.notify
  }

  fn as_any(&self) -> &(dyn core::any::Any + 'static) {
    self
  }
}
