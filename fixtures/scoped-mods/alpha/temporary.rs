fn modify(args:(i32,i32),trace:&Trace)->(i32,i32){trace.borrow_mut().push("tuple");(args.0+1,args.1)}
