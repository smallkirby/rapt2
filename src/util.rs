pub fn split_by_empty_line(s: &str) -> Vec<Vec<String>> {
  let mut result = vec![];
  let mut acc = vec![];

  for line in s.trim().split("\n") {
    if line.trim().len() == 0 {
      if acc.len() != 0 {
        result.push(acc.clone());
      }
      acc.clear();
    } else {
      acc.push(line.into());
    }
  }

  if acc.len() != 0 {
    result.push(acc.clone());
  }

  result
}

pub fn first_numeric(s: &str) -> Option<usize> {
  let s_bytes = s.as_bytes();
  for ix in 0..s.len() {
    if (s_bytes[ix] as char).is_numeric() {
      return Some(ix);
    }
  }

  None
}

pub fn first_non_numeric(s: &str) -> Option<usize> {
  let s_bytes = s.as_bytes();
  for ix in 0..s.len() {
    if !(s_bytes[ix] as char).is_numeric() {
      return Some(ix);
    }
  }

  None
}
