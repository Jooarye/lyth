fn test(abc: i64) i64 {
  return 7 + abc;
}

fn test2(b: i64) i64 {
  if b % 2 == 0 {
    b = test(12);
  }
  return b * test(66);
}