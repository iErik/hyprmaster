//use std::collections::HashMap;
use zaemon::apps::{DesktopEntry, get_apps};

use crate::proxies::AppsProxy;


pub struct AppService<'a> {
  //app_entries: HashMap<String, DesktopEntry>,
  app_list: Vec<DesktopEntry>,
  proxy: Option<AppsProxy<'a>>
}

impl<'a> AppService<'a> {
  pub async fn new (conn: &zbus::Connection) -> Self {
    Self {
      //app_entries
      proxy: AppsProxy::new(conn).await.ok(),
      app_list: vec![]
    }
  }

  pub async fn app_list(&self) -> Vec<DesktopEntry> {
    if let Some(ref p) = self.proxy {
      let apps = p.all_apps().await;

      if !apps.is_err() { return apps.unwrap() }
    }

    get_apps().await
  }
}

