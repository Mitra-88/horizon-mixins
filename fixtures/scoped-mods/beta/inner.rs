fn value(value:&i32,log:&mut Vec<&'static str>){assert_eq!(*value,100);log.push("inner");}
