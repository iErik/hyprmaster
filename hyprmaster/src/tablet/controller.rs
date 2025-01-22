use std::rc::Rc;
use std::sync::Arc;
use tokio::task;

use slint::{spawn_local, ModelRc};

use crate::ui::TabletUIState;
use crate::services::apps::AppService;
use super::model::{SharedAppEntries, LocalAppEntries};


pub struct TabletController<'a> {
  //app_entries: Rc<AppEntries>,
  app_entries: Rc<LocalAppEntries>,
  state: TabletUIState<'a>,
  service: Arc<AppService<'static>>,
}

impl<'a> TabletController<'a> {
  pub fn new (
    state: TabletUIState<'a>,
    service: Arc<AppService<'static>>
  ) -> Self {
    //let app_entries_m = SharedAppEntries::new(service.clone());
    let app_entries_m = LocalAppEntries::new(service.clone());
    let app_entries_rc = Rc::new(app_entries_m);

    state.set_app_entries(
      ModelRc::from(app_entries_rc.clone())
    );

    // TODO: Debt
    /*
    _ = spawn_local(async move {
      handle.populate().await;
    });
    */

    let handle = app_entries_rc.clone();
    state.on_fetch_app_entries(move || {
      let handle = handle.clone();
      _ = spawn_local(async move {
        handle.populate().await
      })
    });

    /*
    let handle = app_entries_rc.clone();
    state.on_filter_app_entries(move |query| {
      let handle = handle.clone();
      _ = spawn_local(async move {
        handle.filter_entries(&query).await
      })
    });
    */

    Self {
      app_entries: app_entries_rc.clone(),
      service,
      state
    }
  }

  fn bind(&self) {

  }
}



