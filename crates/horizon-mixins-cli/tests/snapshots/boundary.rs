use std::cell::RefCell;
use std::rc::Rc;
pub struct Host<'a> {
    pub log: &'a RefCell<Vec<&'static str>>,
}
pub struct Probe(pub Rc<RefCell<usize>>);
impl Drop for Probe {
    fn drop(&mut self) {
        *self.0.borrow_mut() += 1;
    }
}
impl<'a> Host<'a> {
    pub fn run(&mut self, fail: bool, _probe: Probe) -> Result<i32, &'static str> {
        {
            fn head(host: &mut Host<'_>) {
                host.log.borrow_mut().push("alpha");
                support::touch();
            }
            head(&mut *self)
        };
        {
            fn head(host: &mut Host<'_>) {
                host.log.borrow_mut().push("beta");
                support::touch();
            }
            head(&mut *self)
        };
        let work = || if fail { Err("failure") } else { Ok(3) };
        let value = work()?;
        self.log.borrow_mut().push("body");
        Ok(value)
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn preserved_closure_errors_and_drop() {
        for fail in [false, true] {
            let log = RefCell::new(Vec::new());
            let dropped = Rc::new(RefCell::new(0));
            let result = Host { log: &log }.run(fail, Probe(dropped.clone()));
            assert_eq!(result, if fail { Err("failure") } else { Ok(3) });
            assert_eq!(* dropped.borrow(), 1);
            assert_eq!(
                * log.borrow(), if fail { vec!["alpha", "beta"] } else { vec!["alpha",
                "beta", "body"] }
            );
        }
    }
}
pub(crate) mod support {
    pub fn touch() {}
}
