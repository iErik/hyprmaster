use std::boxed::Box;
use std::error::Error;


mod theme;
mod utils;
mod tablet;
mod services;

pub mod ui {
  slint::include_modules!();
}


use ui::*;
use tablet::TabletController;


#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
  let main_window = MainWindow::new()?;
  let services = services::Services::new().await?;

  _ = TabletController::new(
    main_window.global::<TabletUIState>(),
    services.clone()
  ).await;

  main_window.run()
    .map_err(|e| Box::new(e) as Box<dyn Error>)
}
