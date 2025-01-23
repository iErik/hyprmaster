use std::io::ErrorKind as IOErr;
use std::error::Error as STDErr;
use std::collections::HashMap;
use std::env::var;


use zbus::{
  interface,
  object_server::SignalEmitter as Emitter
};

use tokio::net::UnixStream;


pub(crate) struct HyprlandInterface {

}

#[interface(name = "org.hypr.Hyprmaster.Hyprland")]
impl HyprlandInterface {
  #[zbus(signal)]
  async fn workspace(
    e: &Emitter<'_>,
    workspace_name: String,
    workspace_id: String
  ) -> zbus::Result<()>;

  #[zbus(signal)]
  async fn focused_monitor(
    e: &Emitter<'_>,
    monitor_name: String,
    workspace_name: String,
    workspace_id: String
  ) -> zbus::Result<()>;

  #[zbus(signal)]
  async fn active_window(
    e: &Emitter<'_>,
    window_class: String,
    window_title: String,
    window_address: String
  ) -> zbus::Result<()>;

  #[zbus(signal)]
  async fn fullscreen(
    e: &Emitter<'_>,
    status: bool
  ) -> zbus::Result<()>;

  #[zbus(signal)]
  async fn monitor_removed(
    e: &Emitter<'_>,
    monitor_name: String
  ) -> zbus::Result<()>;

  #[zbus(signal)]
  async fn monitor_added(
    e: &Emitter<'_>,
    monitor_id: String,
    monitor_name: String,
    monitor_description: String
  ) -> zbus::Result<()>;

  #[zbus(signal)]
  async fn create_workspace(
    e: &Emitter<'_>,
    workspace_name: String,
    workspace_id: String
  ) -> zbus::Result<()>;

  #[zbus(signal)]
  async fn destroy_workspace(
    e: &Emitter<'_>,
    workspace_id: String,
    workspace_name: String,
  ) -> zbus::Result<()>;

  #[zbus(signal)]
  async fn move_workspace(
    e: &Emitter<'_>,
    workspace_id: String,
    workspace_name: String,
    monitor_name: String
  ) -> zbus::Result<()>;

  #[zbus(signal)]
  async fn rename_workspace(
    e: &Emitter<'_>,
    workspace_id: String,
    new_name: String
  ) -> zbus::Result<()>;

  #[zbus(signal)]
  async fn active_special(
    e: &Emitter<'_>,
    workspace_name: String,
    monitor_name: String
  ) -> zbus::Result<()>;

  #[zbus(signal)]
  async fn active_layout(
    e: &Emitter<'_>,
    keyboard_name: String,
    layout_name: String
  ) -> zbus::Result<()>;

  #[zbus(signal)]
  async fn open_window(
    e: &Emitter<'_>,
    window_address: String,
    workspace_name: String,
    window_class: String,
    window_title: String
  ) -> zbus::Result<()>;

  #[zbus(signal)]
  async fn close_window(
    e: &Emitter<'_>,
    window_address: String
  ) -> zbus::Result<()>;

  #[zbus(signal)]
  async fn move_window(
    e: &Emitter<'_>,
    window_address: String,
    workspace_id: String,
    workspace_name: String,
  ) -> zbus::Result<()>;

  #[zbus(signal)]
  async fn open_layer(
    e: &Emitter<'_>,
    namespace: String
  ) -> zbus::Result<()>;

  #[zbus(signal)]
  async fn close_layer(
    e: &Emitter<'_>,
    namespace: String
  ) -> zbus::Result<()>;

  #[zbus(signal)]
  async fn submap(
    e: &Emitter<'_>,
    submap_name: String
  ) -> zbus::Result<()>;

  #[zbus(signal)]
  async fn change_floating_mode(
    e: &Emitter<'_>,
    window_address: String,
    floating: bool
  ) -> zbus::Result<()>;

  #[zbus(signal)]
  async fn urgent(
    e: &Emitter<'_>,
    window_address: String
  ) -> zbus::Result<()>;

  #[zbus(signal)]
  async fn screencast(
    e: &Emitter<'_>,
    active: bool,
    owner: ScreencastOwner
  ) -> zbus::Result<()>;

  #[zbus(signal)]
  async fn window_title(
    e: &Emitter<'_>,
    window_address: String,
    window_title:String
  ) -> zbus::Result<()>;

  #[zbus(signal)]
  async fn toggle_group(
    e: &Emitter<'_>,
    destroyed: bool,
    window_addresses: Vec<String>
  ) -> zbus::Result<()>;

  #[zbus(signal)]
  async fn move_into_group(
    e: &Emitter<'_>,
    window_address: String
  ) -> zbus::Result<()>;

  #[zbus(signal)]
  async fn move_outof_group(
    e: &Emitter<'_>,
    window_address: String
  ) -> zbus::Result<()>;

  #[zbus(signal)]
  async fn ignore_group_lock(
    e: &Emitter<'_>,
    state: bool
  ) -> zbus::Result<()>;

  #[zbus(signal)]
  async fn lock_groups(
    e: &Emitter<'_>,
    state: bool
  ) -> zbus::Result<()>;

  #[zbus(signal)]
  async fn config_reloaded(e: &Emitter<'_>) -> zbus::Result<()>;

  #[zbus(signal)]
  async fn pin(
    e: &Emitter<'_>,
    window_address: String,
    pinned: bool
  ) -> zbus::Result<()>;
}


impl HyprlandInterface {
  pub fn new() -> Self {
    Self { }
  }

  pub async fn listen(
    conn: &zbus::Connection
  ) -> Result<(), Box<dyn STDErr>> {
    let rtm_dir = var("XDG_RUNTIME_DIR")?;
    let his_dir = var("HYPRLAND_INSTANCE_SIGNATURE")?;

    let sock_src = format!(
      "{rtm_dir}/hypr/{his_dir}/.socket2.sock");

    let sock = UnixStream::connect(sock_src).await?;

    let emit = conn.object_server()
      .interface::<_, HyprlandInterface>("/hyprland").await?;


    loop {
      sock.readable().await?;

      let mut buf = [0; 4096];

      match sock.try_read(&mut buf) {
        Ok(0) => break,
        Err(ref e) if e.kind() == IOErr::WouldBlock => {
          continue;
        },
        Err(e) => {
          println!("Received error: {:#?}", e);
          return Err(e.into());
        },
        _ => ()
      }

      let evs = map_events_string(&buf)?;

      for (name, args) in &evs {
        let args = args.clone().unwrap_or(vec![]);

        let res = match name.as_str() {
          "workspace" => {
            let name = gets(&args, 0);
            let id = match evs.get("workspacev2") {
              Some(Some(a))=> gets(a, 0),
              _=> "".to_string()
            };

            emit.workspace(name, id).await
          },
          "focusedmon" => {
            let m_name = gets(&args, 0);
            let w_name = gets(&args, 1);
            let w_id = match evs.get("focusedmonv2") {
              Some(Some(a))=> gets(a, 0),
              _ => "".to_string()
            };

            emit.focused_monitor(m_name, w_name, w_id).await
          },
          "activewindow" => {
            let w_class = gets(&args, 0);
            let w_title = gets(&args, 1);
            let w_addr  = match evs.get("activewindowv2") {
              Some(Some(a)) => gets(a, 0),
              _ => "".to_string()
            };

            emit.active_window(
              w_class,
              w_title,
              w_addr
            ).await
          }
          "fullscreen" => {
            let status = match gets(&args, 0).as_str() {
              "1" => true,
              _ => false
            };

            emit.fullscreen(status).await
          },
          "monitorremoved" => {
            let w_name = gets(&args, 0);
            emit.monitor_removed(w_name).await
          },
          "monitoraddedv2" => {
            let m_id = gets(&args, 0);
            let m_name = gets(&args, 1);
            let m_desc = gets(&args, 2);

            emit.monitor_added(m_id, m_name, m_desc).await
          },
          "createworkspacev2" => {
            let w_id = gets(&args, 0);
            let w_name = gets(&args, 1);

            emit.create_workspace(w_id, w_name).await
          },
          "destroyworkspacev2" => {
            let w_id = gets(&args, 0);
            let w_name = gets(&args, 1);

            emit.destroy_workspace(w_id, w_name).await
          },
          "moveworkspacev2" => {
            let w_id = gets(&args, 0);
            let w_name = gets(&args, 1);
            let m_name = gets(&args, 2);

            emit.move_workspace(w_id, w_name, m_name).await
          },
          "renameworkspace" => {
            let w_id = gets(&args, 0);
            let new_name = gets(&args, 1);

            emit.rename_workspace(w_id, new_name).await
          },
          "activespecial" => {
            let w_name = gets(&args, 0);
            let m_name = gets(&args, 1);

            emit.active_special(w_name, m_name).await
          },
          "activelayout" => {
            let k_name = gets(&args, 0);
            let l_name = gets(&args, 1);

            emit.active_layout(k_name, l_name).await
          },
          "openwindow" => {
            let addr   = gets(&args, 0);
            let w_name = gets(&args, 1);
            let class  = gets(&args, 2);
            let title  = gets(&args, 3);

            emit.open_window(
              addr,
              w_name,
              class,
              title
            ).await
          },
          "closewindow" => {
            let addr = gets(&args, 0);
            emit.close_window(addr).await
          },
          "movewindowv2" => {
            let wn_addr = gets(&args, 0);
            let w_id    = gets(&args, 1);
            let w_name  = gets(&args, 2);

            emit.move_window(wn_addr, w_id, w_name).await
          },
          "openlayer" => {
            let namespace = gets(&args, 0);
            emit.open_layer(namespace).await
          },
          "closelayer" => {
            let namespace = gets(&args, 0);
            emit.close_layer(namespace).await
          },
          "submap" => {
            let s_name = gets(&args, 0);
            emit.submap(s_name).await
          },
          "changefloatingmode" => {
            let wn_addr  = gets(&args, 0);
            let floating = match gets(&args, 1).as_str() {
              "1" => true,
              _ => false
            };

            emit
              .change_floating_mode(wn_addr, floating).await
          },
          "urgent" => {
            let wn_addr = gets(&args, 0);
            emit.urgent(wn_addr).await
          },
          "screencast" => {
            let active = match gets(&args, 0).as_str() {
              "1" => true,
              _   => false,
            };

            let owner = match gets(&args, 1).as_str() {
              "0" => ScreencastOwner::Monitor,
              _   => ScreencastOwner::Window,
            };

            emit.screencast(active, owner).await
          },
          "windowtitlev2" => {
            let wn_addr  = gets(&args, 0);
            let wn_title = gets(&args, 1);

            emit.window_title(wn_addr, wn_title).await
          },
          "togglegroup" => {
            let destroyed = match gets(&args, 0).as_str() {
              "0" => true,
              _   => false
            };

            let wn_addrs: Vec<String> = gets(&args, 1)
              .split(',')
              .map(|s| s.to_string())
              .collect();

            emit.toggle_group(destroyed, wn_addrs).await
          },
          "moveintogroup" => {
            let addrs = gets(&args, 0);
            emit.move_into_group(addrs).await
          },
          "moveoutofgroup" => {
            let addrs = gets(&args, 0);
            emit.move_outof_group(addrs).await
          },
          "ignoregrouplock" => {
            let state = match gets(&args, 0).as_str() {
              "1" => true,
              _   => false
            };

            emit.ignore_group_lock(state).await
          },
          "lockgroups" => {
            let state = match gets(&args, 0).as_str() {
              "1" => true,
              _   => false
            };

            emit.lock_groups(state).await
          },
          "configreloaded" => {
            emit.config_reloaded().await
          },
          "pin" => {
            let addrs = gets(&args, 0);
            let pinned = match gets(&args, 1).as_str() {
              "1" => true,
              _   => false
            };

            emit.pin(addrs, pinned).await
          },
          _ => Ok(())
        };

        match res {
          Err(e) => println!(
            "Signal dispatch error: {:#?}",
            e
          ),
          _ => ()
        }
      }

      println!("Here, events: {:#?}", evs);
    }

    Ok(())
  }
}

// gets (get_safe)
fn gets(v: &Vec<String>, i: usize) -> String {
  match v.get(i) {
    Some(i) => i.clone(),
    None => String::from("")
  }
}

struct HEvent {
  name: String,
  args: Option<Vec<String>>
}

type HEvMap = HashMap<String, Option<Vec<String>>>;

fn map_events_string(input: &[u8]) ->
  Result<HEvMap, Box<dyn STDErr>>
{
  let mut map: HEvMap = HashMap::new();

  String::from_utf8(Vec::from(input))?
    .trim_end_matches(['\0', '\n', ' '])
    .split('\n')
    .for_each(|s| {
      let s: Vec<&str> = s.split(">>").collect();

      match s.len() {
        0 => return,
        1 => map.insert(s[0].to_string(), None),
        _ => map.insert(s[0].to_string(), Some(s[1]
          .split(',')
          .map(|s| s.to_string())
          .collect()))
      };
    });

  Ok(map)
}

fn parse_events_string(input: &[u8]) -> Vec<HEvent> {
  let text = String::from_utf8(Vec::from(input))
    .unwrap_or("".to_string());

  text
    .trim_end_matches(['\0', '\n', ' '])
    .split('\n')
    .map(|s| {
      let s: Vec<&str> = s.split(">>").collect();

      match s.len() {
        0 => None,
        1 => Some(HEvent {
          name: s[0].to_string(),
          args: None
        }),
        _ => Some(HEvent {
          name: s[0].to_string(),
          args: Some(s[1]
            .split(',')
            .map(|s| s.to_string())
            .collect())
        })
      }
    })
    .filter(|e| e.is_some())
    .map(|e| e.unwrap())
    .collect()
}

#[derive(
  serde::Deserialize,
  serde::Serialize,
  zvariant::Type
)]
enum ScreencastOwner {
  Monitor,
  Window
}

/*
enum HyprlandEvent {
  WorkspaceChanged {
    worspace_name: &str,
    workspace_id: &str
  },

  FocusedMonitor {
    monitor_name: &str,
    workspace_name: &str,
    workspace_id: &str
  },

  ActiveWindow {
    window_class: &str,
    window_title: &str,
    window_address: &str
  },

  Fullscreen {
    is_fullscreen: bool
  },

  MonitorRemoved {
    monitor_name: &str
  },

  MonitorAdded {
    monitor_name: &str,
    monitor_id: &str,
    monitor_description: &str
  },

  CreateWorkspace {
    workspace_name: &str,
    workspace_id: &str
  },

  DestroyWorkspace {
    workspace_name: &str,
    workspace_id: &str
  },

  MoveWorkspace {
    workspace_name: &str,
    workspace_id: &str,
    monitor_name: &str,
  },

  RenameWorkspace {
    workspace_id: &str,
    new_name: &str
  },

  ActiveSpecial {
    workspace_name: &str,
    monitor_name: &str
  },

  ActiveLayout {
    keyboard_name: &str,
    layout_name: &str
  },

  OpenWindow {
    window_address: &str,
    workspace_name: &str,
    window_class: &str,
    window_title: &str
  },

  CloseWindow {
    window_address: &str
  },

  MoveWindow {
    window_address: &str,
    workspace_name: &str,
    workspace_id: &str
  },

  OpenLayer {
    namespace: &str
  },

  CloseLayer {
    namespace: &str
  },

  Submap {
    submap_name: &str
  },

  ChangeFloatingMode {
    window_address: &str,
    floating: bool
  },

  Urgent {
    window_address: &str
  },

  Screencast {
    state: u8,
    owner: u8
  },

  WindowTitle {
    window_address: &str,
    window_title: &str
  },

  ToggleGroup {
    state: u8,
    window_addresses: Vec<&str>
  },

  MoveIntoGroup {
    window_address: &str
  },

  MoveOutOfGroup {
    window_address: &str
  },

  IgnoreGroupLock {
    state: bool
  },

  LockGroups {
    state: bool
  },

  ConfigReloaded,

  Pin {
    window_address: &str,
    pin_state: bool
  }
}
*/
