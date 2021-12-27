/*
 This file implements parse of sources.list file
*/

use super::{error::SourceError, source::Source, source::*};

use std::collections::HashSet;
use std::str::FromStr;

fn parse_line(line: &str) -> Result<HashSet<Source>, SourceError> {
  assert!(!line.contains('\n'));

  // remove comments
  let comment_position = line
    .trim()
    .split_whitespace()
    .position(|part| part.starts_with('#'));
  let mut parts: Vec<&str> = line.trim().split_whitespace().collect();
  if let Some(ix) = comment_position {
    if ix == 0 {
      return Ok(HashSet::new());
    };
    parts = parts[0..ix].to_vec();
  }
  if parts.is_empty() {
    return Ok(HashSet::new());
  } else if parts.len() < 4 {
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

  let url: String = parts[1].into();
  let distro: String = parts[2].into();
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

  Ok(
    components
      .iter()
      .map(|component| Source {
        archive_type: archive_type.clone(),
        url: url.clone(),
        distro: distro.clone(),
        component: component.clone(),
      })
      .collect(),
  )
}

pub fn parse_lines(content: &str) -> Result<HashSet<Source>, SourceError> {
  let mut sources = HashSet::new();
  for line in content.lines() {
    match parse_line(line) {
      Ok(s) => {
        sources.extend(s);
      }
      Err(err) => return Err(err),
    }
  }

  Ok(sources)
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn parse_single_line() {
    // check if normal single line entry is correctly parsed
    let answer = Source::from(
      ArchivedType::DEB,
      "http://jp.archive.ubuntu.com/ubuntu/",
      "focal",
      vec![Component::MAIN, Component::RESTRICTED],
    );
    let line = "deb http://jp.archive.ubuntu.com/ubuntu/ focal main restricted";
    let parsed = parse_line(line).unwrap();
    assert_eq!(answer, parsed);

    let empty: HashSet<Source> = HashSet::new();
    // check if empty line is correctly parsed
    let line = "";
    let parsed = parse_line(line).unwrap();
    assert_eq!(empty, parsed);

    // check if normal line with comment is correctly parsed
    let line = "deb http://jp.archive.ubuntu.com/ubuntu/ focal main restricted # this is comment ";
    let parsed = parse_line(line).unwrap();
    assert_eq!(answer, parsed);

    // check if invalid line can't be parsed
    let line = "deb http://jp.archive.ubuntu.com/ubuntu/ focal # main restricted";
    let parsed = parse_line(line);
    assert_eq!(parsed.is_err(), true);
  }

  #[test]
  fn parse_multi_lines() {
    let s1 = Source::from(
      ArchivedType::DEBSRC,
      "http://archive.ubuntu.com/ubuntu",
      "focal",
      vec![Component::MAIN, Component::RESTRICTED],
    );
    let s2 = Source::from(
      ArchivedType::DEB,
      "http://jp.archive.ubuntu.com/ubuntu/",
      "focal",
      vec![Component::MAIN, Component::RESTRICTED],
    );
    let s3 = Source::from(
      ArchivedType::DEBSRC,
      "http://jp.archive.ubuntu.com/ubuntu/",
      "focal",
      vec![
        Component::MAIN,
        Component::RESTRICTED,
        Component::MULTIVERSE,
        Component::UNIVERSE,
      ],
    );
    let answers: Vec<Source> = vec![s1, s2, s3].into_iter().flatten().collect();
    let answers_set: HashSet<Source> = answers.into_iter().collect();

    let lines = "
      deb-src http://archive.ubuntu.com/ubuntu focal main restricted #Added by software-properties

      # See http://help.ubuntu.com/community/UpgradeNotes for how to upgrade to
      # newer versions of the distribution.
      deb http://jp.archive.ubuntu.com/ubuntu/ focal main restricted
      deb-src http://jp.archive.ubuntu.com/ubuntu/ focal restricted multiverse universe main #Added by software-properties
    ";

    let sources = parse_lines(lines).unwrap();
    assert_eq!(answers_set, sources);
  }
}
