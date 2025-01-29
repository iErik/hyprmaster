use std::io::{self, ErrorKind as IOErr};
use std::error::Error as STDErr;
use std::collections::HashMap;
use std::env::var;

use tokio::{
  sync::broadcast::{self, Sender, Receiver},
  net::UnixStream
};

use zbus::{
  interface,
  zvariant::Type,
  object_server::SignalEmitter as Emitter
};



pub(crate) type HyprSender = Sender<HyprlandEvent>;
pub(crate) type HyprReceiver = Receiver<HyprlandEvent>;

pub(crate) struct HyprlandInterface { }

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
  async fn config_reloaded(
    e: &Emitter<'_>
  ) -> zbus::Result<()>;

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

  pub fn spawn_listener() ->
    (HyprSender, HyprReceiver)
  {
    let (sx, rx) = broadcast::channel(4);
    let sender = sx.clone();

    tokio::spawn(async move {
      let sock = match connect_hypr_sock().await {
        Err(e) => {
          println!(
            "Could not connect to Hyprland socket2: {:#?}",
            e
          );
          return
        },
        Ok(s) => s
      };

      loop {
        match sock.readable().await {
          Err(_) => break,
          _ => ()
        };

        let mut buf = [0; 4096];

        match sock.try_read(&mut buf) {
          Ok(0) => break,
          Err(ref e) if e.kind() == IOErr::WouldBlock => {
            continue;
          },
          Err(e) => {
            println!(
              "Hypr listener received error: {:#?}", e);
          },
          _ => ()
        }

        let evs = map_events_string(&buf).unwrap();
        let events = parse_hypr_events(&evs);

        for ev in events {
          match sender.send(ev) {
            Err(e) => {
              println!(
                "Failed to send HyprlandEvent: {:#?}", e
              );
            },
            _ => ()
          };
        }
      }
    });

    (sx, rx)
  }

  pub async fn listen(
    conn: &zbus::Connection,
    mut receiver: HyprReceiver
  ) ->
    Result<(), Box<dyn STDErr>>
  {
    let emit = conn.object_server()
      .interface::<_, HyprlandInterface>("/hyprland")
      .await?;

    while let Ok(ev) = receiver.recv().await {
      let res = match ev {
        HyprlandEvent::WorkspaceChanged {
          workspace_name,
          workspace_id
        } => emit.workspace(workspace_name, workspace_id),

        HyprlandEvent::FocusedMonitor {
          monitor_name: m_name,
          workspace_name: w_name,
          workspace_id: w_id
        } => emit.focused_monitor(m_name, w_name, w_id),

        HyprlandEvent::ActiveWindow {
          window_class: class,
          window_title: title,
          window_address: addr
        } => emit.active_window(class, title, addr),

        HyprlandEvent::Fullscreen {
          is_fullscreen
        } => emit.fullscreen(is_fullscreen),

        HyprlandEvent::MonitorRemoved {
          monitor_name: m_name
        } => emit.monitor_removed(m_name),

        HyprlandEvent::MonitorAdded {
          monitor_name: name,
          monitor_id: id,
          monitor_description: desc
        } => emit.monitor_added(id, name, desc),

        HyprlandEvent::CreateWorkspace {
          workspace_name: name,
          workspace_id: id
        } => emit.create_workspace(name, id),

        HyprlandEvent::DestroyWorkspace {
          workspace_name: name,
          workspace_id: id
        } => emit.destroy_workspace(id, name),

        HyprlandEvent::MoveWorkspace {
          workspace_name: name,
          workspace_id: id,
          monitor_name: m_name
        } => emit.move_workspace(id, name, m_name),

        HyprlandEvent::RenameWorkspace {
          workspace_id: id,
          new_name: name
        } => emit.rename_workspace(id, name),

        HyprlandEvent::ActiveSpecial {
          workspace_name: w_name,
          monitor_name: m_name
        } => emit.active_special(w_name, m_name),

        HyprlandEvent::ActiveLayout {
          keyboard_name: k_name,
          layout_name: l_name
        } => emit.active_layout(k_name, l_name),

        HyprlandEvent::OpenWindow {
          window_address: addr,
          workspace_name: w_name,
          window_class: class,
          window_title: title
        } => emit.open_window(addr, w_name, class, title),

        HyprlandEvent::CloseWindow {
          window_address
        } => emit.close_window(window_address),

        HyprlandEvent::MoveWindow {
          window_address: addr,
          workspace_name: w_name,
          workspace_id: w_id
        } => emit.move_window(addr, w_id, w_name),

        HyprlandEvent::OpenLayer {
          namespace
        } => emit.open_layer(namespace),

        HyprlandEvent::CloseLayer {
          namespace
        } => emit.close_layer(namespace),

        HyprlandEvent::Submap {
          submap_name
        } => emit.submap(submap_name),

        HyprlandEvent::ChangeFloatingMode {
          window_address: addr,
          floating: status
        } => emit.change_floating_mode(addr, status),

        HyprlandEvent::Urgent {
          window_address
        } => emit.urgent(window_address),

        HyprlandEvent::Screencast {
          active,
          owner
        } => emit.screencast(active, owner),

        HyprlandEvent::WindowTitle {
          window_address: addr,
          window_title: title
        } => emit.window_title(addr, title),

        HyprlandEvent::ToggleGroup {
          destroyed,
          window_addresses: addrs,
        } => emit.toggle_group(destroyed, addrs),

        HyprlandEvent::MoveIntoGroup {
          window_address: addr
        } => emit.move_into_group(addr),

        HyprlandEvent::MoveOutOfGroup {
          window_address: addr
        } => emit.move_outof_group(addr),

        HyprlandEvent::IgnoreGroupLock {
          state
        } => emit.ignore_group_lock(state),

        HyprlandEvent::LockGroups {
          state
        } => emit.lock_groups(state),

        HyprlandEvent::ConfigReloaded =>
          emit.config_reloaded(),

        HyprlandEvent::Pin {
          window_address: addr,
          pinned
        } => emit.pin(addr, pinned)
      };

      match res.await {
        Err(e) => println!(
          "Failed to emit hyprland signal: {:#?}", e),
        _ => ()
      };
    }

    Ok(())
  }

  pub async fn old_listen(
    conn: &zbus::Connection
  ) -> Result<(), Box<dyn STDErr>> {
    let rtm_dir = var("XDG_RUNTIME_DIR")?;
    let his_dir = var("HYPRLAND_INSTANCE_SIGNATURE")?;

    let sock_src = format!(
      "{rtm_dir}/hypr/{his_dir}/.socket2.sock");

    let sock = UnixStream::connect(sock_src).await?;

    let emit = conn.object_server()
      .interface::<_, HyprlandInterface>("/hyprland")
      .await?;


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
    }

    Ok(())
  }
}



async fn connect_hypr_sock() ->
  Result<UnixStream, io::Error>
{
  let rtm_dir = var("XDG_RUNTIME_DIR").unwrap();
  let his_dir = var("HYPRLAND_INSTANCE_SIGNATURE")
    .expect(concat!(
        "HYPRLAND_INSTANCE_SIGNATURE ",
        "environment variable is not set!"
    ));

  let sock_src = format!(
    "{rtm_dir}/hypr/{his_dir}/.socket2.sock");

  let sock = UnixStream::connect(sock_src).await;
  sock
}

fn parse_hypr_events(evs: &HEvMap) -> Vec<HyprlandEvent> {
  let mut events: Vec<HyprlandEvent> = vec![];

  for (name, args) in evs {
    let args = args.clone().unwrap_or(vec![]);

    let event = match name.as_str() {
      "workspace" => Some(
        HyprlandEvent::WorkspaceChanged {
          workspace_name: gets(&args, 0),
          workspace_id: match evs.get("workspacev2") {
            Some(Some(a))=> gets(a, 0),
            _ => "".to_string()
          }
        }),
      "focusedmon" => Some(
        HyprlandEvent::FocusedMonitor {
          monitor_name: gets(&args, 0),
          workspace_name: gets(&args, 1),
          workspace_id: match evs.get("focusedmonv2") {
            Some(Some(a))=> gets(a, 0),
            _ => "".to_string()
          }
        }),
      "activewindow" => Some(
        HyprlandEvent::ActiveWindow {
          window_class: gets(&args, 0),
          window_title: gets(&args, 1),
          window_address: match evs.get("activewindowv2") {
            Some(Some(a)) => gets(a, 0),
            _ => "".to_string()
          }
      }),
      "fullscreen" => Some(HyprlandEvent::Fullscreen {
        is_fullscreen: match getstr(&args, 0) {
          "1" => true,
          _ => false
        }
      }),
      "monitorremoved" => Some(
        HyprlandEvent::MonitorRemoved {
          monitor_name: gets(&args, 0)
        }),
      "monitoraddedv2" => Some(
        HyprlandEvent::MonitorAdded {
          monitor_id: gets(&args, 0),
          monitor_name: gets(&args, 1),
          monitor_description: gets(&args, 2)
        }),
      "createworkspacev2" => Some(
        HyprlandEvent::CreateWorkspace {
          workspace_id: gets(&args, 0),
          workspace_name: gets(&args, 1)
        }),
      "destroyworkspacev2" => Some(
        HyprlandEvent::DestroyWorkspace {
          workspace_id: gets(&args, 0),
          workspace_name: gets(&args, 1),
        }),
      "moveworkspacev2" => Some(
        HyprlandEvent::MoveWorkspace {
          workspace_id: gets(&args, 0),
          workspace_name: gets(&args, 1),
          monitor_name: gets(&args, 2)
        }),
      "renameworkspace" => Some(
        HyprlandEvent::RenameWorkspace {
          workspace_id: gets(&args, 0),
          new_name: gets(&args, 1)
        }),
      "activespecial" => Some(
        HyprlandEvent::ActiveSpecial {
          workspace_name: gets(&args, 0),
          monitor_name: gets(&args, 1)
        }),
      "activelayout" => Some(
        HyprlandEvent::ActiveLayout {
          keyboard_name: gets(&args, 0),
          layout_name: gets(&args, 1)
        }),
      "openwindow" => Some(HyprlandEvent::OpenWindow {
        window_address: gets(&args, 0),
        workspace_name: gets(&args, 1),
        window_class: gets(&args, 2),
        window_title: gets(&args, 3)
      }),
      "closewindow" => Some(HyprlandEvent::CloseWindow {
        window_address: gets(&args, 0)
      }),
      "movewindowv2" => Some(HyprlandEvent::MoveWindow {
        window_address: gets(&args, 0),
        workspace_id: gets(&args, 1),
        workspace_name: gets(&args, 2)
      }),
      "openlayer" => Some(HyprlandEvent::OpenLayer {
        namespace: gets(&args, 0)
      }),
      "closelayer" => Some(HyprlandEvent::CloseLayer {
        namespace: gets(&args, 0)
      }),
      "submap" => Some(HyprlandEvent::Submap {
        submap_name: gets(&args, 0)
      }),
      "changefloatingmode" => Some(
        HyprlandEvent::ChangeFloatingMode {
          window_address: gets(&args, 0),
          floating: match getstr(&args, 1) {
            "1" => true,
            _ => false
          }
        }),
      "urgent" => Some(HyprlandEvent::Urgent {
        window_address: gets(&args, 0)
      }),
      "screencast" => Some(HyprlandEvent::Screencast {
        active: match getstr(&args, 0) {
          "1" => true,
          _   => false,
        },
        owner: match getstr(&args, 1) {
          "0" => ScreencastOwner::Monitor,
          _   => ScreencastOwner::Window,
        }
      }),
      "windowtitlev2" => Some(
        HyprlandEvent::WindowTitle {
          window_address: gets(&args, 0),
          window_title: gets(&args, 1)
        }),
      "togglegroup" => Some(HyprlandEvent::ToggleGroup {
        destroyed: match getstr(&args, 0) {
          "0" => true,
          _   => false
        },
        window_addresses: gets(&args, 1)
          .split(',')
          .map(|s| s.to_string())
          .collect()
      }),
      "moveintogroup" => Some(
        HyprlandEvent::MoveIntoGroup {
          window_address: gets(&args, 0)
        }),
      "moveoutofgroup" => Some(
        HyprlandEvent::MoveOutOfGroup {
          window_address: gets(&args, 0)
        }),
      "ignoregrouplock" => Some(
        HyprlandEvent::IgnoreGroupLock {
          state: match getstr(&args, 0) {
            "1" => true,
            _   => false
          }

        }),
      "lockgroups" => Some(HyprlandEvent::LockGroups {
        state: match getstr(&args, 0) {
          "1" => true,
          _   => false
        }
      }),
      "configreloaded" => Some(
        HyprlandEvent::ConfigReloaded
      ),
      "pin" => Some(HyprlandEvent::Pin {
        window_address: gets(&args, 0),
        pinned: match getstr(&args, 1) {
          "1" => true,
          _   => false
        }
      }),
      _ => None
    };

    if event.is_some() {
      events.push(event.unwrap());
    }
  }

  events
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


struct HEvent {
  name: String,
  args: Option<Vec<String>>
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

fn getstr(v: &Vec<String>, i: usize) -> &str {
  match v.get(i) {
    Some(i) => i.as_str(),
    None => ""
  }
}

fn gets(v: &Vec<String>, i: usize) -> String {
  match v.get(i) {
    Some(i) => i.clone(),
    None => String::from("")
  }
}


#[derive(
  serde::Deserialize,
  serde::Serialize,
  Type,
  Clone,
  Debug
)]
pub enum ScreencastOwner {
  Monitor,
  Window
}

#[derive(Debug, Clone)]
pub enum HyprlandEvent {
  WorkspaceChanged {
    workspace_name: String,
    workspace_id: String
  },

  FocusedMonitor {
    monitor_name: String,
    workspace_name: String,
    workspace_id: String
  },

  ActiveWindow {
    window_class: String,
    window_title: String,
    window_address: String
  },

  Fullscreen {
    is_fullscreen: bool
  },

  MonitorRemoved {
    monitor_name: String
  },

  MonitorAdded {
    monitor_name: String,
    monitor_id: String,
    monitor_description: String
  },

  CreateWorkspace {
    workspace_name: String,
    workspace_id: String
  },

  DestroyWorkspace {
    workspace_name: String,
    workspace_id: String
  },

  MoveWorkspace {
    workspace_name: String,
    workspace_id: String,
    monitor_name: String,
  },

  RenameWorkspace {
    workspace_id: String,
    new_name: String
  },

  ActiveSpecial {
    workspace_name: String,
    monitor_name: String
  },

  ActiveLayout {
    keyboard_name: String,
    layout_name: String
  },

  OpenWindow {
    window_address: String,
    workspace_name: String,
    window_class: String,
    window_title: String
  },

  CloseWindow {
    window_address: String
  },

  MoveWindow {
    window_address: String,
    workspace_name: String,
    workspace_id: String
  },

  OpenLayer {
    namespace: String
  },

  CloseLayer {
    namespace: String
  },

  Submap {
    submap_name: String
  },

  ChangeFloatingMode {
    window_address: String,
    floating: bool
  },

  Urgent {
    window_address: String
  },

  Screencast {
    active: bool,
    owner: ScreencastOwner
  },

  WindowTitle {
    window_address: String,
    window_title: String
  },

  ToggleGroup {
    destroyed: bool,
    window_addresses: Vec<String>
  },

  MoveIntoGroup {
    window_address: String
  },

  MoveOutOfGroup {
    window_address: String
  },

  IgnoreGroupLock {
    state: bool
  },

  LockGroups {
    state: bool
  },

  ConfigReloaded,

  Pin {
    window_address: String,
    pinned: bool
  }
}

mod events {
  pub struct WorkspaceChanged {
    workspace_name: String,
    workspace_id: String
  }

  pub struct FocusedMonitor {
    monitor_name: String,
    workspace_name: String,
    workspace_id: String
  }

  pub struct ActiveWindow {
    window_class: String,
    window_title: String,
    window_address: String
  }

  pub struct Fullscreen {
    is_fullscreen: bool
  }

  pub struct MonitorRemoved {
    monitor_name: String
  }

  pub struct MonitorAdded {
    monitor_name: String,
    monitor_id: String,
    monitor_description: String
  }

  pub struct CreateWorkspace {
    workspace_name: String,
    workspace_id: String
  }

  pub struct DestroyWorkspace {
    workspace_name: String,
    workspace_id: String
  }

  pub struct MoveWorkspace {
    workspace_name: String,
    workspace_id: String,
    monitor_name: String,
  }

  pub struct RenameWorkspace {
    workspace_id: String,
    new_name: String
  }

  pub struct ActiveSpecial {
    workspace_name: String,
    monitor_name: String
  }

  pub struct ActiveLayout {
    keyboard_name: String,
    layout_name: String
  }

  pub struct OpenWindow {
    window_address: String,
    workspace_name: String,
    window_class: String,
    window_title: String
  }

  pub struct CloseWindow {
    window_address: String
  }

  pub struct MoveWindow {
    window_address: String,
    workspace_name: String,
    workspace_id: String
  }

  pub struct OpenLayer {
    namespace: String
  }

  pub struct CloseLayer {
    namespace: String
  }

  pub struct Submap {
    submap_name: String
  }

  pub struct ChangeFloatingMode {
    window_address: String,
    floating: bool
  }

  pub struct Urgent {
    window_address: String
  }

  pub struct Screencast {
    active: bool,
    owner: super::ScreencastOwner
  }

  pub struct WindowTitle {
    window_address: String,
    window_title: String
  }

  pub struct ToggleGroup {
    destroyed: bool,
    window_addresses: Vec<String>
  }

  pub struct MoveIntoGroup {
    window_address: String
  }

  pub struct MoveOutOfGroup {
    window_address: String
  }

  pub struct IgnoreGroupLock {
    state: bool
  }

  pub struct LockGroups {
    state: bool
  }

  pub struct Pin {
    window_address: String,
    pinned: bool
  }
}
