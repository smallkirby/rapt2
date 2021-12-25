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
