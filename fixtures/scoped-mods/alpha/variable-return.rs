fn modify(value:i32,health:&i32,log:&mut Vec<&'static str>)->i32{log.push("return");value+*health}
