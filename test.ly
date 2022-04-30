fn test(abc: i64) i64 {
  let x: i64 = 5;
  return x * 7 + abc;
}

fn test2() i64 {
  return 1 * test(777);
}