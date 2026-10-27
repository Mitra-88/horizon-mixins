fn around(original: impl FnOnce() -> i32) -> i32 { let _ = original; 19 }
