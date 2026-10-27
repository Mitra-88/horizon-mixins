fn modify(args:(String,i32,Trace))->(String,i32,Trace){let(value,number,trace)=args;trace.borrow_mut().push("tuple-beta");(value+"b",number*2,trace)}
