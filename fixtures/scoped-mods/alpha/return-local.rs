fn modify(value:i32,extra:&i32,log:&mut Vec<&'static str>)->i32{log.push("return");value+*extra}
