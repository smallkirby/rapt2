pub mod update;

#[derive(Debug)]
pub enum SubCommand {
  UPDATE,
  UPGRADE,
  INSTALL,
  REMOVE,
  PURGE,
  SEARCH,
  SHOW,
}
