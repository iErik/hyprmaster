use std::sync::Arc;
use std::error::Error;

pub mod apps;
pub mod tablet;

use apps::AppService;
use tablet::TabletService;


#[derive(Clone)]
pub struct Services<'a> {
  connection: zbus::Connection,
  tablet: Arc<TabletService<'a>>,
  apps: Arc<AppService<'a>>
}

impl<'a> Services<'a> {
  pub async fn new() -> Result<Self, Box<dyn Error>> {
    let conn = zbus::Connection::session().await?;

    let (tablet_srv, apps_srv) = tokio::join!(
      TabletService::new(&conn),
      AppService::new(&conn)
    );

    Ok(Self {
      connection: conn,
      tablet: Arc::new(tablet_srv),
      apps: Arc::new(apps_srv)
    })
  }

  pub async fn init_tablet(&self) ->
    Result<(), Box<dyn Error>>
  {
    self.tablet.init().await
  }

  pub async fn init_apps(&self) ->
    Result<(), Box<dyn Error>>
  {
    self.apps.init().await
  }

  pub fn tablet(&self) -> Arc<TabletService<'a>> {
    self.tablet.clone()
  }

  pub fn apps(&self) -> Arc<AppService<'a>> {
    self.apps.clone()
  }
}
