fn around(value:String,number:i32,trace:Trace,original:impl FnOnce(String,i32,Trace)->String)->String{trace.borrow_mut().push("wrap");original(value+"w",number,trace)+"!"}
