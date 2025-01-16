mod icons;
mod packages;
mod theme;
mod apps;
mod tablet;
mod xdg;
mod utils;

pub mod ui {
  slint::include_modules!();
}

use ui::*;
use apps::SharedModel;

use std::sync::Arc;
use std::rc::Rc;
use slint::{self, ModelRc};


use std::thread;
use dbus::blocking::Connection;
use dbus::arg::Array;
use std::time::Duration;


pub type DesktopEntryTU = (
  String,
  String,
  String,
  String,
  String,
  String,
  String,
  bool,
  bool,
);


fn main() -> Result<(), slint::PlatformError> {
  let main_window = Arc::new(MainWindow::new().unwrap());
  let tablet_state = main_window.global::<TabletUIState>();

  let shared_model = SharedModel::new();
  let shared_model_rc = Rc::new(shared_model);

  let shared_model_rc_a = Rc::clone(&shared_model_rc);
  let shared_model_rc_b = Rc::clone(&shared_model_rc_a);
  let shared_mrc  = ModelRc::from(shared_model_rc.clone());


  tablet_state.set_app_entries(shared_mrc);

  tablet_state.on_fetch_app_entries(move || {
    println!("App entries requested.");
    let handle = shared_model_rc_a.clone();
    //let handle = handle.upgrade().unwrap();

    _ = slint::spawn_local(async move {
      handle.populate_async().await
    }).unwrap();
  });

  tablet_state.on_filter_app_entries(move |query| {
    let handle = shared_model_rc_b.clone();
    _ = slint::spawn_local(async move {
      handle.filter_entries(&query).await;
    }).unwrap();
  });

  thread::spawn(|| {
    let conn = Connection::new_session().unwrap();
    let proxy = conn.with_proxy(
      "org.hypr.hyprmaster",
      "/hello",
      Duration::from_millis(5000)
    );

    let (response, ): (Vec<DesktopEntryTU>,) = proxy.method_call(
      "org.hypr.hyprmaster",
      "AppList",
      ()).unwrap();

    //println!("Response: {}", response);

    for item in response {
      println!("Item: {:#?}", item);
    }
  });

  main_window.run()
}
