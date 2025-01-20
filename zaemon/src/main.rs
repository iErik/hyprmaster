
use std::error::Error;
use zbus::connection;


mod dconf;
mod objects;

use objects::{AppsObject, IconsObject};




#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
  let icons = IconsObject::new();
  let apps = AppsObject::new();

  let conn = connection::Builder::session()?
    .name("org.hypr.Hyprmaster")?
    .serve_at("/icons", icons)?
    .serve_at("/apps", apps)?
    .build()
    .await?;

  _ = tokio::join!(
    AppsObject::listen(&conn)
  );

  Ok(())
}
