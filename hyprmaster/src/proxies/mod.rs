use zbus::{proxy, Result};
use zaemon::DesktopEntry;

#[proxy(
  interface = "org.hypr.Hyprmaster.Apps",
  default_service = "org.hypr.Hyprmaster",
  default_path = "/apps"
)]
pub trait AppsProxy {
  async fn all_apps(&self) -> Result<Vec<DesktopEntry>>;
}

