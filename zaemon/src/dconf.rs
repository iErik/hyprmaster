use std::process::{Command, Output};

pub fn set(
  key: &str,
  prop: &str,
  value: &str
) -> Result<(), String> {
  let mut cmd = Command::new("gsettings");
  cmd.args(&["set", key, prop, value]);

  match cmd.output() {
    Ok(_) => Ok(()),
    Err(_) => Err("Unable to set key".to_string()),
  }
}

pub fn get(
  key: &str,
  prop: &str
) -> Result<String, String> {
  let mut cmd = Command::new("gsettings");
  cmd.args(&["get", key, prop]);

  match cmd.output() {
    Ok(Output { stdout, .. }) => {
      let stdout_string = String::from_utf8(stdout)
        .unwrap()
        .trim()
        .replace("'", "")
        .replace("@as", "")
        .replace("\"", "")
        .replace("\n", "");

      Ok(stdout_string.to_string())
    }
    Err(_) => Err("Unable to get key".to_string()),
  }
}


pub fn reset(key: &str, prop: &str) -> Result<(), String> {
  let mut cmd = Command::new("gsettings");
  cmd.args(&["reset", key, prop]);

  match cmd.output() {
    Ok(_) => Ok(()),
    Err(_) => Err("Unable to reset key".to_string()),
  }
}

pub fn interface(prop: &str) -> Result<String, String>{
  get("org.gnome.desktop.interface", prop)
}

