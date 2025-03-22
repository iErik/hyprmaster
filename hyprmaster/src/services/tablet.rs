use std::error::Error;
use std::cell::RefCell;
use std::rc::Rc;

use slint::SharedString;
use zbus::{Connection, proxy};
pub use zaemon::tablet::TabletBindings;

use crate::ui::AppBinding;


#[proxy(
  interface = "org.hypr.Hyprmaster.Tablet",
  default_service = "org.hypr.Hyprmaster",
  default_path = "/tablet",
  gen_async = true
)]
pub trait Tablet {
  #[zbus(property)]
  fn presets(&self) -> zbus::Result<Vec<String>>;

  #[zbus(property)]
  fn bindings(&self) -> zbus::Result<TabletBindings>;

  fn add_preset(&self) -> zbus::Result<()>;

  fn remove_preset(&self) -> zbus::Result<()>;
}


pub type Presets = Vec<SharedString>;
pub type Bindings = Vec<AppBinding>;


pub type PresetsRc = Rc<RefCell<Presets>>;
//pub type BindingsRc = Rc<RefCell<Bindings>>;
pub type BindingsRc = Rc<RefCell<TabletBindings>>;

pub struct TabletService<'a> {
  proxy: Option<TabletProxy<'a>>,
  presets: PresetsRc,
  bindings: BindingsRc,
}

impl<'a> TabletService<'a> {
  pub async fn new(conn: &Connection) -> Self {
    Self {
      proxy: TabletProxy::new(conn).await.ok(),
      presets: PresetsRc::default(),
      bindings: BindingsRc::default()
    }
  }

  pub async fn init(&self) ->
    Result<(), Box<dyn Error>>
  {
    if !self.proxy.is_some() { return Ok(()) }

    let proxy = &self.proxy.as_ref().unwrap();

    let (presets, bindings) = tokio::try_join!(
      proxy.presets(),
      proxy.bindings(),
    )?;

    let mut spresets = self.presets.borrow_mut();
    *spresets = presets
      .iter().map(|s| SharedString::from(s))
      .collect();

    let mut sbindings = self.bindings.borrow_mut();
    *sbindings = bindings.clone();
    //*sbindings = map_bindings(&bindings);

    Ok(())
  }

  pub fn presets(&self) -> PresetsRc {
    self.presets.clone()
  }

  pub fn bindings(&self) -> BindingsRc {
    self.bindings.clone()
  }
}

/*
fn map_bindings(bindings: &TabletBindings) ->
  Vec<AppBinding>
{
  bindings
    .iter()
    .map(|(k, v)| AppBinding {
      app_name: k.into(),
      preset: v.into()
    })
    .collect()
}
*/
