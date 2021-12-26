use dirs::home_dir;
use std::path::PathBuf;

pub struct Context {
  pub rapt_dir: PathBuf,   // base dir of rapt2
  pub list_dir: PathBuf,   // package list dir
  pub source_dir: PathBuf, // source list dir
  pub dpkg_dir: PathBuf,   // dpkg base dir
}

impl Default for Context {
  fn default() -> Self {
    let home = home_dir().unwrap();
    let rapt_dir = home.join("rapt2");
    let list_dir = rapt_dir.join("lists");
    let source_dir = rapt_dir.join("sources");

    let dpkg_dir = PathBuf::from("/var/lib/dpkg");

    Context {
      rapt_dir,
      list_dir,
      source_dir,
      dpkg_dir,
    }
  }
}
