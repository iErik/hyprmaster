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
  let conn = connection::Builder::session()?
    .name("org.hypr.Hyprmaster")?
    .serve_at("/hyprland", HyprlandInterface::new())?
    .serve_at("/tablet", TabletInterface::new())?
    .serve_at("/icons", IconsObject::new())?
    .serve_at("/apps", AppsObject::new())?
    .max_queued(300)
    .build()
    .await?;

  let (hsx, hrx) = HyprlandInterface::spawn_listener();

  _ = tokio::join!(
    AppsObject::listen(&conn),
    TabletInterface::listen(&conn, hsx.subscribe()),
    HyprlandInterface::listen(&conn, hrx)
  );

  Ok(())
}
