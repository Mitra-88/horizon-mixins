fn modify(args:(Token,i32))->(Token,i32){args.0.0.borrow_mut().push("tuple");(args.0,args.1+1)}
