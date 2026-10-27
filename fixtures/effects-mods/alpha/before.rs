fn before(cancel:bool,trace:&Trace)->::core::ops::ControlFlow<i32> { mark(trace,"before"); if cancel { ::core::ops::ControlFlow::Break(41) } else { ::core::ops::ControlFlow::Continue(()) } }
