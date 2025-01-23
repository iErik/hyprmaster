use std::fmt::{self, Display, Formatter};
use std::time::Duration;
use std::path::PathBuf;
use std::error::Error;
use std::boxed::Box;


use notify_debouncer_full::{
  notify::{
    EventKind as EvKind,
    RecursiveMode
  },
  new_debouncer
};

use tokio::{
  task::JoinSet,
  sync::mpsc::unbounded_channel as channel
};

use freedesktop_entry_parser::{
  parse_entry,
  Entry as EntryFile
};

use zbus::{
  interface,
  ObjectServer,
  message::Message,
  names::MemberName,
  object_server::{Interface, SignalEmitter, DispatchResult},
};

use walkdir::WalkDir;
use async_stream::stream;
use futures_core::stream::Stream;
use futures_util::{stream::StreamExt, pin_mut};

use crate::utils::notify::DebouncedSender;
use super::IconsObject;



#[derive(
  serde::Deserialize,
  serde::Serialize,
  zvariant::Type,
  Default,
  Debug,
  Clone,
  PartialEq,
  Eq,
)]
pub struct DesktopEntry {
  pub entry_path  :String,
  pub exec        :String,
  pub name        :String,

  pub icon_name   :String,
  pub icon_path   :String,

  pub wm_class    :String,
  pub description :String,

  pub no_display  :bool,
  pub terminal    :bool,
}


impl PartialOrd for DesktopEntry {
  fn partial_cmp(&self, other: &Self) ->
    Option<std::cmp::Ordering>
  {
    self.name.partial_cmp(&other.name)
  }
}

impl Ord for DesktopEntry {
  fn cmp(&self, other: &Self) -> std::cmp::Ordering {
    self.name.cmp(&other.name)
  }
}


pub(crate) struct AppsObject {
  cache: Vec<DesktopEntry>,
}

impl AppsObject {
  pub fn new() -> Self {
    Self { cache: vec![] }
  }

  pub async fn trigger_cache_reset (
    conn: &zbus::Connection
  ) -> Result<(), Box<dyn Error>>
  {
    let member = MemberName::try_from("ResetCache")?;
    let msg = Message::method_call("/apps", member.clone())?
      .interface("org.hypr.Hyprmaster.Apps")?
      .build(&())?;

    let appref = conn.object_server()
     .interface::<_, AppsObject>("/apps").await?;
    let mut appref = appref.get_mut().await;

    let res = appref.call_mut(
      &conn.object_server(),
      &conn,
      &msg,
      member,
    );

    match res {
      DispatchResult::NotFound => {
        Err(Box::<dyn Error>::from(ErrNotFound))
      },
      DispatchResult::RequiresMut => {
        Err(Box::<dyn Error>::from(ErrRequiresMut))
      },
      DispatchResult::Async(v) => {
        v.await.map_err(|e| Box::new(e) as Box<dyn Error>)
      }
    }
  }

  pub async fn listen(conn: &zbus::Connection) ->
    Result<(), Box<dyn Error>>
  {
    // Initial cache reset
    Self::trigger_cache_reset(conn).await?;

    let (sx, mut rx) = channel();
    let sender = DebouncedSender(sx);
    let timeframe = Duration::from_secs(1);

    let mut debouncer = new_debouncer(
      timeframe, None, sender)?;

    app_lookup_dirs()
      .iter()
      .for_each(|p| {
        if let Ok(_) = p.try_exists() {
          let res = debouncer.watch(
            &p,
            RecursiveMode::NonRecursive
          );

          if res.is_err() {
            _ = debouncer.unwatch(&p);
          }
        }
      });

    while let Some(db) = rx.recv().await {
      let ev = db.event;

      match ev.kind {
          EvKind::Create(_) |
          EvKind::Modify(_) |
          EvKind::Remove(_) => {
            println!("Event received: {:#?}", ev);
            println!("Rebuilding cache: {:#?}", ev);

            let res = Self::trigger_cache_reset(conn).await;

            if res.is_err() {
              println!("ResetCache error: {:#?}", res);
            }
          }
          _ => ()
      }
    }

    Ok(())
  }
}

// TODO:
// - Filter method
// - Proper icon_path resolution
// - persistent caching
// - apps changed/added/removed signals
#[interface(name = "org.hypr.Hyprmaster.Apps")]
impl AppsObject {
  fn all_apps(&self) -> Vec<DesktopEntry> {
    println!("App list requested!");
    self.cache.clone()
  }

  async fn reset_cache(
    &mut self,
    #[zbus(signal_emitter)]
    sig_emitter: SignalEmitter<'_>,
    #[zbus(object_server)]
    srv: &ObjectServer
  ) -> zbus::fdo::Result<()> {
    let icns_intr = srv
      .interface::<_, IconsObject>("/icons").await
      .expect("Should fetch interface");
    let mut icns_intr = icns_intr.get_mut().await;

    self.cache.clear();
    let app_stream = get_apps_stream();
    pin_mut!(app_stream);

    while let Some(mut app) = app_stream.next().await {
      match app.icon_name.trim().is_empty() {
        true => self.cache.push(app),
        false => {
          app.icon_path = icns_intr
            .get_icon(app.icon_name.as_str()).await;
          self.cache.push(app);
        }
      }
    }

    self.cache.sort();
    sig_emitter.app_list_changed().await
      .expect("Should emit app_list_changed signal");

    Ok(())
  }

  #[zbus(signal)]
  async fn app_list_changed(
    emitter: &SignalEmitter<'_>
  ) -> zbus::Result<()>;
}



trait EntryAttrGetters {
  fn get_bool(&self, attr: &str) -> bool;
  fn get_str(&self, attr: &str) -> String;
  fn wm_class(&self) -> String;
  fn icon_name(&self) -> String;
}

impl EntryAttrGetters for EntryFile {
  fn get_bool(&self, attr: &str) -> bool {
    let sec = self.section("Desktop Entry");

    match sec.attr(attr).unwrap_or("") {
      "true" | "True" => true,
      _ => false
    }
  }

  fn get_str(&self, attr: &str) -> String {
    self.section("Desktop Entry")
      .attr(attr)
      .unwrap_or("")
      .into()
  }

  fn wm_class(&self) -> String {
    let sec = self.section("Desktop Entry");

    if sec.has_attr("StartupWMClass") {
      return self.get_str("StartupWMClass")
    }

    if !sec.has_attr("Exec") {
      return "".to_string()
    }

    let exec: String = self.get_str("Exec")
      .split(' ').nth(0).unwrap_or("".into()).to_string();

    match PathBuf::from(&exec).exists() {
      true => PathBuf::from(exec)
        .file_stem().unwrap()
        .to_string_lossy()
        .to_string(),
      _ => String::from("")
    }
  }

  fn icon_name(&self) -> String {
    self.get_str("Icon")
      .replace('"', "")
      .replace('\'', "")
  }
}


pub fn app_lookup_dirs() -> Vec<PathBuf> {
  let mut lookup_dirs: Vec<String> =
    std::env::var("XDG_DATA_DIRS")
    .unwrap_or("/usr/share".to_string())
    .split(':')
    .map(|s| match s.ends_with("/") {
      true => format!("{s}applications"),
      false => format!("{s}/applications"),
    })
    .collect();

  lookup_dirs.push("/usr/share/applications".to_string());
  let home_dir = std::env::var("HOME").unwrap();
  let home_path = PathBuf::from(home_dir);
  let local_share_apps = home_path
    .join(".local/share/applications")
    .into_os_string()
    .into_string()
    .unwrap();

  lookup_dirs.push(local_share_apps);
  lookup_dirs.sort();
  lookup_dirs.dedup();


  let lookup_dirs: Vec<PathBuf>  = lookup_dirs
    .iter()
    .map(|pathstr| PathBuf::from(pathstr.clone()))
    .filter(|p| p.try_exists().unwrap())
    .collect();

  lookup_dirs
}

pub async fn get_apps() -> Vec<DesktopEntry> {
  let lookup_dirs = app_lookup_dirs();

  let mut apps: Vec<DesktopEntry> = Vec::new();
  let mut set: JoinSet<Option<DesktopEntry>> = JoinSet::new();

  for dir in lookup_dirs {
    if !dir.exists() { continue; }

    for entry in WalkDir::new(dir.clone()) {
      if entry.is_err() { continue; }
      let entry = entry.unwrap().clone();

      set.spawn_blocking(move || {
        let path = entry.path();

        if path.extension()? != "desktop" { return None }

        make_entry(path.to_path_buf())
      });
    }
  }

  while let Some(res) = set.join_next().await {
    if res.is_err() { continue }

    match res.unwrap() {
      Some(app_entry) => apps.push(app_entry),
      _ => continue
    }
  }

  apps.sort();
  apps
}


fn get_apps_stream() -> impl Stream<Item = DesktopEntry>
{
  let lookup_dirs = app_lookup_dirs();
  let mut set: JoinSet<Option<DesktopEntry>> = JoinSet::new();

  for dir in lookup_dirs {
    if !dir.exists() { continue; }

    for entry in WalkDir::new(dir.clone()) {
      if entry.is_err() { continue; }
      let entry = entry.unwrap().clone();

      set.spawn_blocking(move || {
        let path = entry.path();

        if path.extension()? != "desktop" { return None }

        make_entry(path.to_path_buf())
      });
    }
  }

  stream! {
    while let Some(res) = set.join_next().await {
      if res.is_err() { continue }

      match res.unwrap() {
        Some(app_entry) => yield app_entry,
        _ => continue
      }
    }
  }
}

fn make_entry(path: PathBuf) -> Option<DesktopEntry> {
  let entry: EntryFile = match parse_entry(path.to_str()?) {
    Ok(e) => e,
    _ => return None
  };

  if !entry.has_section("Desktop Entry") {
    return None;
  }

  Some(DesktopEntry {
    entry_path: path.clone()
      .into_os_string()
      .into_string()
      .unwrap(),

    exec: entry.get_str("Exec"),
    name: entry.get_str("Name"),

    icon_name: entry.icon_name(),
    icon_path: String::from(""),
    /*
    icon_path: match entry.icon_name().trim() {
      "" => String::from(""),
      _ => get_icon_sync(&entry.icon_name())
        .unwrap_or("".to_string())
    },
    */

    description: entry.get_str("Comment"),
    wm_class: entry.wm_class(),

    no_display: entry.get_bool("NoDisplay"),
    terminal: entry.get_bool("Terminal")
  })
}


#[derive(Debug)]
struct ErrNotFound;
impl Error for ErrNotFound {}

#[derive(Debug)]
struct ErrRequiresMut;
impl Error for ErrRequiresMut {}

impl Display for ErrNotFound {
  fn fmt (&self, f: &mut Formatter) -> fmt::Result {
    write!(f, "ErrNotFound")
  }
}

impl Display for ErrRequiresMut {
  fn fmt (&self, f: &mut Formatter) -> fmt::Result {
    write!(f, "ErrRequiresMut")
  }
}

