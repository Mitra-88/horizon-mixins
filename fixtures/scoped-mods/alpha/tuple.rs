fn modify(args:(String,i32,Trace),bonus:&i32)->(String,i32,Trace){let(value,number,trace)=args;trace.borrow_mut().push("tuple-alpha");(value+"a",number+*bonus,trace)}
