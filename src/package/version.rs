/*
 This file defines Version of deb packages.
*/

use crate::util::*;

use std::cmp;

static NON_ALPHA_BIAS: u8 = 0x7F;
static EMPTY_PSEUDO_ASCII_CODE: u8 = 0x1;
static CHILDA_ASCII_CODE: u8 = 0x7E;
static CHILDA_PSEUDO_ASCII_CODE: u8 = 0x0;

/*
  [epoch:]upstream-version[-debian-revision]

  - epoch:
     unsigned 1-digit integer. Optional if 0.
  - upstream-version:
     r/[A-Za-z0-9\.\+\-:\~]+
     `-` is allowed only if `debian-revision` exsits.
  - debian-revision:
     Optional.
     r/[A-Za-z0-9\+\.\~]
     Regarded as smaller than minimum number if this field doesn't exist.
*/

#[derive(Debug, PartialEq, Hash, Eq, Clone)]
pub struct Version {
  epoch: u64, // XXX not sure it is used when ordering and comparision.
  upstream_version: String,
  debian_revision: String,
}

impl Version {
  #[allow(clippy::result_unit_err)]
  pub fn from(s: &str) -> Result<Self, ()> {
    match s.rfind('-') {
      Some(last_hyphen) => match s.find(':') {
        Some(colon_ix) => Ok(Self {
          epoch: s[0..colon_ix].to_string().parse().unwrap(),
          upstream_version: s[colon_ix + 1..last_hyphen].into(),
          debian_revision: s[last_hyphen + 1..].into(),
        }),
        None => Ok(Self {
          epoch: 0,
          upstream_version: s[..last_hyphen].into(),
          debian_revision: s[last_hyphen + 1..].into(),
        }),
      },
      None => Ok(Self {
        epoch: 0,
        upstream_version: s.into(),
        debian_revision: String::new(),
      }),
    }
  }
}

impl std::fmt::Display for Version {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    write!(
      f,
      "{}:{}-{}",
      self.epoch, self.upstream_version, self.debian_revision
    )
  }
}

impl Default for Version {
  fn default() -> Self {
    Self {
      epoch: 0,
      upstream_version: "".into(),
      debian_revision: "".into(),
    }
  }
}

impl PartialOrd for Version {
  fn partial_cmp(&self, other: &Self) -> Option<cmp::Ordering> {
    match compare_version(&self.upstream_version, &other.upstream_version) {
      core::cmp::Ordering::Equal => {}
      ord => return Some(ord),
    }
    Some(compare_version(
      &self.debian_revision,
      &other.debian_revision,
    ))
  }
}

fn compare_version(s1: &str, s2: &str) -> cmp::Ordering {
  let mut s1 = s1;
  let mut s2 = s2;
  while !s1.is_empty() || !s2.is_empty() {
    // check alphabetic (plus symbols) string
    let s1_str = match first_numeric(s1) {
      Some(ix) => {
        let ret = &s1[0..ix];
        s1 = &s1[ix..];
        ret
      }
      None => {
        let ret = s1;
        s1 = "";
        ret
      }
    };
    let s2_str = match first_numeric(s2) {
      Some(ix) => {
        let ret = &s2[0..ix];
        s2 = &s2[ix..];
        ret
      }
      None => {
        let ret = s2;
        s2 = "";
        ret
      }
    };
    match compare_version_alphasymbols(s1_str, s2_str) {
      cmp::Ordering::Equal => {}
      ord => return ord,
    };

    // compare numeric string
    let s1_str = match first_non_numeric(s1) {
      Some(ix) => {
        let ret = &s1[0..ix];
        s1 = &s1[ix..];
        ret
      }
      None => {
        let ret = s1;
        s1 = "";
        ret
      }
    };
    let s2_str = match first_non_numeric(s2) {
      Some(ix) => {
        let ret = &s2[0..ix];
        s2 = &s2[ix..];
        ret
      }
      None => {
        let ret = s2;
        s2 = "";
        ret
      }
    };
    match compare_version_number(s1_str, s2_str) {
      cmp::Ordering::Equal => {}
      ord => return ord,
    };
  }

  cmp::Ordering::Equal
}

#[derive(Debug, Default, PartialEq, Eq, Hash, Clone)]
pub struct VersionComp {
  pub version: Version,
  pub operator: VersionCompOperator,
}

impl std::fmt::Display for VersionComp {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    write!(f, "{} {}", self.operator, self.version)
  }
}

impl VersionComp {
  #[allow(clippy::result_unit_err)]
  pub fn from(s: &str) -> Result<Self, ()> {
    match s.find(' ') {
      Some(ix) => {
        let ope_str = &s[0..ix];
        let version_str = &s[ix + 1..];
        Ok(Self {
          version: Version::from(version_str).unwrap(),
          operator: VersionCompOperator::from(ope_str),
        })
      }
      None => {
        Ok(Self {
          version: Version::from(s).unwrap(),
          operator: VersionCompOperator::EQ, // XXX
        })
      }
    }
  }

  fn matches(&self, other: &Version) -> bool {
    use VersionCompOperator::*;
    match self.operator {
      GT => &self.version < other,
      GE => &self.version <= other,
      EQ => &self.version == other,
      LT => &self.version > other,
      LE => &self.version >= other,
      ANY => true,
    }
  }
}

#[derive(Debug, PartialEq, Eq, Hash, Clone)]
pub enum VersionCompOperator {
  GT, // >, >>
  GE, // >=
  EQ, // =
  LT, // <, <<
  LE, // <=
  ANY,
}

impl std::fmt::Display for VersionCompOperator {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    match self {
      Self::GT => write!(f, ">"),
      Self::GE => write!(f, ">="),
      Self::EQ => write!(f, "="),
      Self::LT => write!(f, ">"),
      Self::LE => write!(f, ">="),
      Self::ANY => write!(f, "<>"),
    }
  }
}

impl Default for VersionCompOperator {
  fn default() -> Self {
    Self::ANY
  }
}

impl VersionCompOperator {
  pub fn from(s: &str) -> Self {
    match s {
      ">" | ">>" => Self::GT,
      ">=" => Self::GE,
      "=" => Self::EQ,
      "<" | "<<" => Self::LT,
      "<=" => Self::LE,
      _ => panic!("unkwnon version comparator: {}", s),
    }
  }
}

/*
 Use customized ASCII lexical order.
   - `~` is smaller than any.
   - alphabets are samaller than non-alphabets other than `~`.
   - Chars in a same group are compared just in lexical order.
*/
fn compare_version_alphasymbols(s1: &str, s2: &str) -> cmp::Ordering {
  let mut s1 = s1.as_bytes().iter();
  let mut s2 = s2.as_bytes().iter();

  while s1.len() != 0 || s2.len() != 0 {
    let c1 = match s1.next() {
      Some(c) => {
        if *c == CHILDA_ASCII_CODE {
          CHILDA_PSEUDO_ASCII_CODE
        } else if (*c as char).is_alphabetic() {
          *c
        } else {
          *c + NON_ALPHA_BIAS
        }
      }
      None => EMPTY_PSEUDO_ASCII_CODE,
    };
    let c2 = match s2.next() {
      Some(c) => {
        if *c == CHILDA_ASCII_CODE {
          CHILDA_PSEUDO_ASCII_CODE
        } else if (*c as char).is_alphabetic() {
          *c
        } else {
          *c + NON_ALPHA_BIAS
        }
      }
      None => EMPTY_PSEUDO_ASCII_CODE,
    };

    match c1.cmp(&c2) {
      cmp::Ordering::Equal => {}
      ord => return ord,
    }
  }

  cmp::Ordering::Equal
}

fn compare_version_number(s1: &str, s2: &str) -> cmp::Ordering {
  let n1: u32 = if s1.is_empty() {
    0
  } else {
    s1.parse().unwrap()
  };
  let n2: u32 = if s2.is_empty() {
    0
  } else {
    s2.parse().unwrap()
  };
  n1.cmp(&n2)
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn test_version_parse() {
    let v1_str = "1:8.1.0+r23-3build2";
    let v2_str = "0.6.55-0ubuntu12~20.04.5";
    let v3_str = "0.0.7+17.10.20170922-0ubuntu1";

    assert_eq!(Version::from(v1_str).is_ok(), true);
    assert_eq!(Version::from(v2_str).is_ok(), true);
    assert_eq!(Version::from(v3_str).is_ok(), true);
  }

  #[test]
  fn test_version_ordering() {
    let v1 = Version::from("1:8.1.0+r23-3build2").unwrap();
    let v2 = Version::from("1:8.1.0+r23-3build2").unwrap();
    let v3 = Version::from("0-~~").unwrap();
    let v4 = Version::from("0-~~a").unwrap();
    let v5 = Version::from("0-~").unwrap();
    let v6 = Version::from("0").unwrap();
    let v7 = Version::from("0-a").unwrap();
    let v8 = Version::from("1:8.1.0+r25-3build2").unwrap();
    let v9 = Version::from("1:8.1.0+r25-3build9").unwrap();

    assert_eq!(v1 == v2, true);
    assert_eq!(v3 < v4 && v4 < v5 && v5 < v6 && v6 < v7, true);
    assert_eq!(v8 < v9, true);
  }

  #[test]
  fn version_comp_comparision() {
    let v1 = VersionComp::from("> 1.0.0").unwrap();
    let v2 = VersionComp::from("= 2.0.0").unwrap();
    let v3 = VersionComp::from(">> 1.0.0").unwrap();
    let v4 = VersionComp::from("<< 2.0.0").unwrap();
    assert_eq!(v1.matches(&v2.version), true);
    assert_eq!(v1.matches(&v3.version), false);
    assert_eq!(v2.matches(&v4.version), true);
    assert_eq!(v2.matches(&v2.version), true);
  }
}
