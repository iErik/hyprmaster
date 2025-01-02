mod icons;
mod packages;
mod theme;
mod apps;
mod tablet;
mod xdg;
mod models;

pub mod ui {
  slint::include_modules!();
}

use ui::*;
use models::app_entries::AppEntriesModel;

use std::sync::Arc;
use std::rc::Rc;
use slint::{self, ModelRc};

fn main() -> Result<(), slint::PlatformError> {
  let main_window = Arc::new(MainWindow::new().unwrap());
  let tablet_state = main_window.global::<TabletUIState>();

  let raw_entries = AppEntriesModel::new();
  let arg = Arc::new(raw_entries);
  let arb = Arc::downgrade(&arg);

  let app_entries = Rc::new(Arc::into_inner(arg).unwrap());
  let model_rc = ModelRc::from(app_entries.clone());
  let app_entries_handle = app_entries.clone();

  let arc = main_window.as_weak();

  tablet_state.set_app_entries(model_rc);

  tablet_state.on_fetch_app_entries(move || {
    let handle = app_entries_handle.clone();

    _ = slint::spawn_local(async move {
      handle.populate_async().await
    });
  });

  std::thread::spawn(move || {
    let handle = arc.clone();

    slint::invoke_from_event_loop(move || {
      let h = handle.upgrade().unwrap();
      let state = h.global::<TabletUIState>();
      let entries = state.get_app_entries();
    })
  });

  main_window.run()
}
