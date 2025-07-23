use std::error::Error;
use zbus::connection;


mod interfaces;
mod dconf;
mod utils;
mod api;

use interfaces::{
  AppsObject,
  IconsObject,
  TabletInterface,
  HyprlandInterface
};


async fn start_dbus() -> Result<(), Box<dyn Error>> {
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
    AppsObject::listen(conn.clone()),
    TabletInterface::listen(conn.clone(), hsx.subscribe()),
    HyprlandInterface::listen(conn.clone(), hrx)
  );

  Ok(())
}


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

  // tokio::join is concurrent but not parallel, meaning
  // that all three of these tasks run on the same thread.
  // If any of them block said thread, the others will have
  // to halt until the thread is freed again.
  _ = tokio::join!(
    tokio::spawn(AppsObject::listen(conn.clone())),
    tokio::spawn(HyprlandInterface
      ::listen(conn.clone(), hrx)),
    tokio::spawn(TabletInterface
      ::listen(conn.clone(), hsx.subscribe()))
  );

  Ok(())
}
