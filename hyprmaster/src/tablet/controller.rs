use std::rc::Rc;

use slint::{spawn_local, ModelRc};
use zbus::proxy;

use crate::ui::TabletUIState;
use super::model::AppEntries;


pub struct TabletController<'a> {
  state: TabletUIState<'a>,
  app_entries: Rc<AppEntries>
}

impl<'a> TabletController<'a> {
  pub fn new (global: TabletUIState<'a>) -> Self {
    let app_entries_m = AppEntries::new();
    let app_entries_rc = Rc::new(app_entries_m);


    global.set_app_entries(
      ModelRc::from(app_entries_rc.clone())
    );

    let handle = app_entries_rc.clone();
    global.on_fetch_app_entries(move || {
      let handle = handle.clone();
      _ = spawn_local(async move {
        handle.populate_async().await
      })
    });

    let handle = app_entries_rc.clone();
    global.on_filter_app_entries(move |query| {
      let handle = handle.clone();
      _ = spawn_local(async move {
        handle.filter_entries(&query).await
      })
    });

    Self {
      app_entries: app_entries_rc.clone(),
      state: global
    }
  }

  fn bind(&self) {

  }
}


#[proxy(
  interface = "org.hypr.Hyprmaster.Apps",
  default_service = "org.hypr.Hyprmaster",
  default_path = "/apps"
)]
trait AppsProxy {
  async fn all_apps(&self) -> zbus::Result<Vec<DesktopEntry>>;
}
