use gomoku::gomoku::SIZE;

pub fn xy_to_point(col: u32, row: u32) -> usize {
  assert!(col < SIZE);
  assert!(row < SIZE);
  (col * SIZE + row) as usize
}

pub fn point_to_xy(point: usize) -> (u32, u32) {
  (point as u32 / SIZE, point as u32 % SIZE)
}

pub fn parse_point(s: &str) -> Option<usize> {
  let mut chars = s.chars();
  let col: u32 = match chars.next() {
    Some(c) if 'A' <= c && c < 'I' => c as u32 - ('A' as u32),
    Some(c) if 'I' < c && c <= 'Z' => c as u32 - ('B' as u32),
    Some(c) if 'a' <= c && c < 'i' => c as u32 - ('a' as u32),
    Some(c) if 'i' < c && c <= 'z' => c as u32 - ('b' as u32),
    _ => {
      return None;
    }
  };

  let mut row = 0;

  for c in chars {
    let digit = match c {
      '0' ... '9' => c as u32 - ('0' as u32),
      _ => return None
    };
    row = row * 10 + digit;
  }

  return Some(xy_to_point(col, row - 1));
}
