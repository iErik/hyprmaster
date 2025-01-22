use zbus::{proxy, Result};
pub use zaemon::apps::DesktopEntry;

#[proxy(
  interface = "org.hypr.Hyprmaster.Apps",
  default_service = "org.hypr.Hyprmaster",
  default_path = "/apps"
)]
pub trait Apps {
  async fn all_apps(&self) -> Result<Vec<DesktopEntry>>;
}

#[proxy(
  interface = "org.hypr.Hyprmaster.Tablet",
  default_service = "org.hypr.Hyprmaster",
  default_path = "/tablet"
)]
pub trait Tablet {
  async fn presets() -> Result<()>;
  async fn app_bindings() -> Result<()>;
  async fn current_preset() -> Result<()>;
}
