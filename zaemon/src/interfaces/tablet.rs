use std::collections::{HashSet, HashMap};
use std::process::Stdio;
use std::time::Duration;
use std::error::Error;
use std::path::Path;
use std::env::var;
use std::fs;

use zbus::{interface, fdo};
use tokio::{
  process::Command,
  io::{BufReader, AsyncBufReadExt},
  sync::mpsc::unbounded_channel as channel
};

use notify_debouncer_full::{
  new_debouncer,
  notify::{self, EventKind as EvKind, RecursiveMode}
};


use super::{HyprReceiver, HyprlandEvent};
use crate::utils::notify::DebouncedSender;
use crate::api::otd::OpenTabletDriver;
use crate::dconf;



pub type TabletBindings = HashMap<String, String>;

pub(crate) struct TabletInterface {
  presets: HashSet<String>,
  bindings: TabletBindings
}

#[interface(name = "org.hypr.Hyprmaster.Tablet")]
impl TabletInterface {
  #[zbus(property)]
  fn presets(&self) -> Vec<String> {
    self.presets.clone().into_iter().collect()
  }

  #[zbus(property)]
  fn bindings(&self) -> TabletBindings {
    self.bindings.clone()
  }

  async fn add_binding(&mut self, app: &str, preset: &str)
    -> fdo::Result<()>
  {
    self.bindings.insert(app.into(), preset.into());
    set_bindings(&self.bindings).await
  }

  async fn modify_binding(
    &mut self,
    app: &str,
    preset: &str
  )
    -> fdo::Result<()>
  {
    if self.bindings.contains_key(app) {
      return Err(fdo::Error::Failed(
        "Binding doesn't exist".into()))
    }

    self.bindings.insert(app.into(), preset.into());
    set_bindings(&self.bindings).await
  }

  async fn remove_binding( &mut self, app: &str)
    -> zbus::fdo::Result<()>
  {
    self.bindings.remove(app);
    set_bindings(&self.bindings).await
  }
}

/**
 *
 */

impl TabletInterface {
  pub fn new() -> Self {
    Self {
      presets: get_presets().unwrap_or(HashSet::new()),
      bindings: get_bindings()
    }
  }

  async fn watch_gsettings(conn: &zbus::Connection) ->
    Result<(), zbus::Error>
  {
    let mut cmd = Command::new("gsettings")
      .stdout(Stdio::piped())
      .arg("monitor")
      .arg("org.hypr.Hyprmaster.tablet")
      .arg("bindings")
      .spawn()?;

    let stdout = cmd.stdout.take()
      .expect("child process did not have stdout handle");
    let mut reader = BufReader::new(stdout).lines();

    let iface = conn.object_server()
      .interface::<_, TabletInterface>("/tablet").await?;

    _ = cmd.wait().await
      .expect("gsettings process encountered an error");

    while let Some(line) = reader.next_line().await? {
      let line          = line.replace("bindings:", "");
      let bindings      = parse_bindings(line.to_string());
      let mut iface_ref = iface.get_mut().await;

      if iface_ref.bindings == bindings { continue }

      iface_ref.bindings = bindings;
      let response = iface_ref
        .bindings_changed(iface.signal_emitter()).await;

      match response {
        Err(e) => eprintln!(
          "Failed to set iface bindings: {:#?}", e),
        Ok(_) => ()
      };
    }

    Ok(())
  }

  // TODO: BIG Debt
  async fn watch_presets(conn: &zbus::Connection)
    //-> Result<(), Box<dyn Error + Send>>
    -> Result<(), zbus::Error>
  {
    let p = get_presets_dir();
    let p = Path::new(&p);

    let (sx, mut rx) = channel();
    let sender = DebouncedSender(sx);
    let mut debouncer = new_debouncer(
      Duration::from_secs(1), None, sender)
      .map_err(|_| zbus::Error::Failure(String::from(
        "Failed to create notify debouncer instance"
      )))?;

    match debouncer.watch(p, RecursiveMode::NonRecursive) {
      Err(e) => {
        println!(
          "Failed to bind tablet presets watcher: {:#?}", e
        );

        return Err(zbus::Error::Failure(String::from(
          "Failed to bind tablet presets observer")))
      },
      _ => ()
    };

    let iface = conn.object_server()
      .interface::<_, TabletInterface>("/tablet").await?;

    while let Some(db) = rx.recv().await {
      let ev = db.event;

      match ev.kind {
        EvKind::Create(k) => {
          if k != notify::event::CreateKind::File {
            continue
          }

          let mut iref = iface.get_mut().await;
          for p in &ev.paths {
            let name = p
              .file_stem().unwrap()
              .to_str().unwrap()
              .to_string();

            iref.presets.insert(name);
          }

          let emitter = iface.signal_emitter();
          _ = iref.presets_changed(emitter).await;
        },
        EvKind::Modify(_) => {
          // Path list has to consist of a list of pairs
          // of paths, otherwise we dunno what to do
          if ev.paths.len() % 2 != 0 {
            continue;
          }

          let items: Vec<String> = ev.paths
            .iter()
            .map(|p| p
              .file_stem().unwrap()
              .to_str().unwrap()
              .to_string())
            .collect();

          let mut change_map = HashMap::new();
          let mut chunks = items.chunks_exact(2);
          let mut iref = iface.get_mut().await;
          let bindings = iref.bindings.clone();

          let mut emit_pr = false;
          let mut emit_bi = false;

          while let Some([from, to]) = chunks.next() {
            if iref.presets.remove(from) {
              emit_pr = true;

              iref.presets.insert(to.to_string());
            }

            change_map.insert(from.clone(), to.clone());
          }

          for (k, v) in bindings {
            if change_map.contains_key(&v) {
              emit_bi = true;

              iref.bindings.insert(
                k,
                change_map[&v].clone()
              );
            }
          }

          let emitter = iface.signal_emitter();

          if emit_pr {
            _ = iref.presets_changed(emitter).await;
          }

          if emit_bi {
            _ = iref.bindings_changed(emitter).await;
          }
        },
        EvKind::Remove(k) => {
          if k != notify::event::RemoveKind::File {
            continue
          }

          let mut iref = iface.get_mut().await;
          let bindings = iref.bindings.clone();
          let items: Vec<String> = ev.paths
            .iter()
            .map(|p| p.
              file_stem().unwrap()
              .to_str().unwrap()
              .to_string())
            .collect();

          let mut emit_pr = false;
          let mut emit_bi = false;

          for i in &items {
            emit_pr = emit_pr || iref.presets.remove(i);
          }

          for (k, v) in bindings {
            if items.contains(&v) {
              emit_bi = true;

              iref.bindings.remove(&k);
            }
          }

          let emitter = iface.signal_emitter();

          if emit_pr {
            _ = iref.presets_changed(emitter).await;
          }

          if emit_bi {
            _ = iref.bindings_changed(emitter).await;
          }
        },
        _ => continue
      }
    }

    Ok(())
  }

  async fn watch_hyprsock(
    conn: &zbus::Connection,
    mut hrx: HyprReceiver
  )
    -> Result<(), zbus::Error>
  {
    let otd = OpenTabletDriver::new();
    let mut last_preset = String::from("");
    let iface = conn.object_server()
      .interface::<_, TabletInterface>("/tablet")
      .await?;
    println!("boom");

    while let Ok(ev) = hrx.recv().await {

      match ev {
        HyprlandEvent::ActiveWindow {
          window_class: wclass,
          ..
        } => {
          let wclass = wclass.to_lowercase();
          let bindings = iface.get().await.bindings.clone();

          if !bindings.contains_key(&wclass) { continue; }
          if bindings[&wclass] == last_preset { continue }

          match otd.apply_preset(&bindings[&wclass]) {
            Ok(_) => {
              last_preset = bindings[&wclass].clone();
            },
            Err(e) => eprintln!(
              "Failed to apply tablet preset: {:#?}", e)
          };
        },
        _  => continue,
      };
    }

    /*
    loop {
      let ev = hrx.recv().await;

      match ev {
        Ok(HyprlandEvent::ActiveWindow {
          window_class: wclass,
          ..
        }) => {
          let wclass = wclass.to_lowercase();
          let bindings = iface.get().await.bindings.clone();

          if !bindings.contains_key(&wclass) { continue; }
          if bindings[&wclass] == last_preset { continue }

          match otd.apply_preset(&bindings[&wclass]) {
            Ok(_) => {
              last_preset = bindings[&wclass].clone();
            },
            Err(e) => eprintln!(
              "Failed to apply tablet preset: {:#?}", e)
          };
        },
        Err(e) => {
          println!("Hyprland socket error: {:#?}", e);
          break;
        }
        _  => continue,
      };
    }
    */


    println!("dropped");

    Ok(())
  }

  pub async fn listen(
    conn: zbus::Connection,
    hrx: HyprReceiver
  ) {
    _ = tokio::join!(
      Self::watch_gsettings(&conn),
      Self::watch_presets(&conn),
      Self::watch_hyprsock(&conn, hrx)
    );

    //Ok(())
  }
}



fn get_presets() -> Option<HashSet<String>> {
 let presets_dir = get_presets_dir();

  let paths = match fs::read_dir(presets_dir) {
    Ok(dirs) => dirs,
    Err(_) => return None
  };

  let mut set = HashSet::new();
  paths
    .map(|d| d.unwrap()
      .path()
      .file_stem()
      .unwrap()
      .to_string_lossy()
      .to_string())
    .for_each(|d| { set.insert(d); });

  Some(set)
}

fn get_presets_dir() -> String {
  let base_dir     = "OpenTabletDriver/Presets";
  let home_dir     = var("HOME").unwrap();
  let xdg_conf_dir = var("XDG_CONFIG_HOME")
    .unwrap_or(format!("{home_dir}/.config"));

  format!("{xdg_conf_dir}/{base_dir}")
}

fn parse_bindings(val: String) -> TabletBindings {
  let mut bindings_map = HashMap::new();

  val
    .trim()
    .strip_suffix(['}', ']']).unwrap()
    .strip_prefix(['{', '[']).unwrap()
    .split(",")
    .for_each(|s| {
      let s: Vec<&str> = s.split(':').collect();

      let key   = s.first().unwrap_or(&"").trim();
      let value = s.last().unwrap_or(&"").trim();

      bindings_map.insert(
        key.to_string(),
        value.to_string()
      );
    });

  bindings_map
}

fn get_bindings() -> TabletBindings {
  let bindings =
    dconf::get("org.hypr.Hyprmaster.tablet", "bindings")
    .unwrap();

  parse_bindings(bindings)
}

async fn set_bindings(to: &TabletBindings) ->
  zbus::fdo::Result<()>
{
  let serialized = serde_json::to_string(to);

  if serialized.is_err() {
    return Err(zbus::fdo::Error::Failed(
      "Failed to serialize bindings".to_string()))
  }

  let result = Command::new("gsettings")
    .stdout(Stdio::null())
    .arg("set")
    .arg("org.hypr.Hyprmaster.tablet")
    .arg("bindings")
    .arg(serialized.unwrap())
    .spawn()
    .map_err(zbus::Error::from)?
    .wait()
    .await;

  match result {
    Ok(r) if r.success() => Ok(()),
    _ => Err(zbus::fdo::Error::Failed(
      "Failed to update dconf value for the binding"
        .to_string()))
  }
}

/*
 * -
 * -> OpenTabletDriver Integration
 * -
 */

fn apply_preset(preset_name: &str) ->
  Result<(),Box<dyn Error>>
{
  let status = std::process::Command::new("otd")
    .arg("applypreset")
    .arg(preset_name)
    .stdout(Stdio::null())
    .spawn();

  match status {
    Ok(_) => Ok(()),
    Err(e) => Err(Box::from(e) as Box<dyn Error>)
  }
}
