use std::rc::Rc;

use slint::{ModelRc, SortModel};

use crate::ui::TabletUIState;

use super::models::{
  AppEntries,
  TabletPresets,
  AppBindings,
};


pub struct TabletController<'a> {
  state: TabletUIState<'a>,
  services: crate::services::Services<'static>,
}

impl<'a> TabletController<'a> {
  pub async fn new(
    state: TabletUIState<'a>,
    services: crate::services::Services<'static>
  ) -> Self {
    let bindings = Rc::new(
      AppBindings::new(services.clone()));
    let presets = Rc::new(
      TabletPresets::new(services.clone()));

    let raw = Rc::new(AppEntries::new(services.clone()));
    let raw_b = raw.clone();

    let ui_app_entries = Rc::new(SortModel::new(raw_b,
      |lhs, rhs| lhs.cmp(rhs)));

    let handle_a = raw.clone();
    state.on_filter_app_entries(move |query| {
      handle_a.filter_entries(&query)
    });

    _ = tokio::join!(
      services.init_tablet(),
      services.init_apps()
    );

    state.set_bindings(ModelRc::from(bindings.clone()));
    state.set_app_entries(ModelRc::from(ui_app_entries));
    state.set_presets(ModelRc::from(presets));

    Self {
      services,
      state
    }
  }
}


