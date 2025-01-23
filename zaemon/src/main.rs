use std::error::Error;
use zbus::connection;


mod interfaces;
mod dconf;
mod utils;

use interfaces::{
  AppsObject,
  IconsObject,
  TabletInterface,
  HyprlandInterface
};



#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
  let icons = IconsObject::new();
  let apps = AppsObject::new();
  let tablet = TabletInterface::new();
  let hyprland = HyprlandInterface::new();

  let conn = connection::Builder::session()?
    .name("org.hypr.Hyprmaster")?
    .serve_at("/hyprland", hyprland)?
    .serve_at("/tablet", tablet)?
    .serve_at("/icons", icons)?
    .serve_at("/apps", apps)?
    .build()
    .await?;

  _ = tokio::join!(
    AppsObject::listen(&conn),
    //TabletInterface::listen()
    HyprlandInterface::listen(&conn)
  );

  Ok(())
}
