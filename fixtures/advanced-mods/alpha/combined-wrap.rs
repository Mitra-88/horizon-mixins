fn around(value: i32, original: impl FnOnce(i32) -> i32) -> i32 { original(value * 2) + 5 }
