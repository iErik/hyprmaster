use std::collections::HashMap;
use std::path::PathBuf;
use std::env::var_os;

use slint_build::{
  compile_with_config,
  CompilerConfiguration as Config
};


fn main() -> Result<(), slint_build::CompileError> {
  let mut ui_dir = PathBuf::from(
    var_os("CARGO_MANIFEST_DIR").unwrap());

  ui_dir.push("ui");

  let config = Config::new()
    .with_include_paths(vec![
      ui_dir.join("icons"),
      ui_dir.join("include"),
    ])
    .with_library_paths(HashMap::from([
      ("ui".into(), ui_dir.clone()),
      ("sections".into(), ui_dir.join("sections")),
      ("widgets".into(), ui_dir.join("widgets"))
    ]));

  compile_with_config("ui/main.slint", config)
}
