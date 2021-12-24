/*
 This file implements parse of sources.list file
*/

use super::{error::SourceError, source::Source, source::*};

use std::str::FromStr;

fn parse_line(line: &str) -> Result<Option<Source>, SourceError> {
  assert!(!line.contains("\n"));

  // remove comments
  let comment_position = line
    .trim()
    .split_whitespace()
    .position(|part| part.starts_with("#"));
  let mut parts: Vec<&str> = line.trim().split_whitespace().collect();
  if let Some(ix) = comment_position {
    if ix == 0 {
      return Ok(None);
    };
    parts = parts[0..ix].to_vec();
  }
  if parts.len() < 4 {
    return Err(SourceError::InvalidFormat { msg: line.into() });
  }

  // collect information
  let archive_type = match ArchivedType::from_str(parts[0]) {
    Ok(t) => t,
    Err(()) => {
      return Err(SourceError::InvalidField {
        field: "ArchivedType".into(),
        value: parts[0].into(),
      })
    }
  };

  let url = parts[1].into();
  let distro = parts[2].into();
  let mut components = vec![];

  for &component_str in &parts[3..] {
    let component = match Component::from_str(component_str) {
      Ok(c) => c,
      Err(()) => {
        return Err(SourceError::InvalidField {
          field: "Component".into(),
          value: component_str.into(),
        })
      }
    };
    components.push(component);
  }

  Ok(Some(Source {
    archive_type,
    url,
    distro,
    components,
  }))
}

pub fn parse_lines(content: &str) -> Result<Vec<Source>, SourceError> {
  let mut sources = vec![];
  for line in content.lines() {
    match parse_line(line) {
      Ok(s) => {
        if let Some(source) = s {
          sources.push(source);
        }
      }
      Err(err) => return Err(err),
    }
  }

  Ok(sources)
}
