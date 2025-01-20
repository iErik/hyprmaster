use std::boxed::Box;
use std::error::Error;

mod icons;
mod packages;
mod theme;
mod tablet;
mod xdg;
mod utils;
mod proxies;
mod services;

pub mod ui {
  slint::include_modules!();
}

use ui::*;
use tablet::TabletController;


#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
  let main_window = MainWindow::new()?;

  let conn = zbus::Connection::session().await?;

  println!("Conn: {:#?}", conn);

  /*
  _ = TabletController::new(
    main_window.global::<TabletUIState>()
  );
  */


  main_window.run()
    .map_err(|e| Box::new(e) as Box<dyn Error>)
}
