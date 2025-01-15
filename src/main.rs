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
use std::rc::{Rc, Weak};
use std::cell::RefCell;
use slint::{self, ModelRc, Model};


fn main() -> Result<(), slint::PlatformError> {
  let main_window = Arc::new(MainWindow::new().unwrap());
  let tablet_state = main_window.global::<TabletUIState>();

  let shared_model = SharedModel::new();
  let shared_model_rc = Rc::new(shared_model);

  let shared_model_rc_a = Rc::clone(&shared_model_rc);
  //let shared_model_rc_b = Rc::new(RefCell::new(shared_model));
  let shared_model_rc_b = Rc::clone(&shared_model_rc_a);

  //let model_cell  = RefCell::new(shared_model);

  //let shared_model_rcl = Rc::new(RefCell::new(shared_model));
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
    //let handle = Rc::downgrade(&shared_model_rc);
    //let mut handle = handle.upgrade().unwrap();
    //let handle = Rc::get_mut(&mut handle);
    let handle = shared_model_rc_b.clone();
    //let handle = handle.borrow_mut();
    //let handle = Rc::get_mut(&mut handle).expect("You suck.");
    //let mut handle = Rc::into_inner(handle).expect("YOU SUCKKKK");

    _ = slint::spawn_local(async move {
      handle.filter_entries(&query).await;
    }).unwrap();
  });

  main_window.run()
}
