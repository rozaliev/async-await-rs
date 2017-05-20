use std::rc::Rc;
use std::cell::RefCell;
use std::ops::{State, Generator};

pub struct Core {
    tasks: Rc<RefCell<Vec<Box<G>>>>,
}
pub struct Handle {
    tasks: Rc<RefCell<Vec<Box<G>>>>,
}

impl Core {
    pub fn new() -> Core {
        Core { tasks: Rc::new(RefCell::new(Vec::new())) }
    }

    pub fn handle(&self) -> Handle {
        Handle { tasks: self.tasks.clone() }
    }


    pub fn run<T, G: Generator<Return = T, Yield = ()>>(&self, mut g: G) -> T {
        loop {
            if let State::Complete(r) = g.resume(()) {
                return r;
            }

            // println!("main task not ready");

            let mut compl = Vec::new();
            let mut tasks = self.tasks.borrow_mut();
            for (i, g) in (*tasks).iter_mut().enumerate() {
                match g.resume() {
                    State::Yielded(()) => continue,
                    State::Complete(()) => {
                        compl.push(i);
                    }
                }
            }

            for i in compl {
                println!("task {} completed", i);
                tasks.remove(i);
            }

        }


    }
}

trait G {
    fn resume(&mut self) -> State<(), ()>;
}

impl<T> G for T
    where T: Generator<Return = (), Yield = ()> + 'static
{
    fn resume(&mut self) -> State<(), ()> {
        self.resume(())
    }
}

impl Handle {
    pub fn spawn<G: Generator<Return = (), Yield = ()> + 'static>(&self, g: G) {
        let mut tasks = self.tasks.borrow_mut();
        tasks.push(Box::new(g));
    }
}




#[macro_export]
macro_rules! await {
    ($e:expr) => ({
        let mut g = $e;
        let ret;
        loop {
            match g.resume(()) {
                State::Yielded(()) => yield,
                State::Complete(r) => {
                    ret = r;
                    break
                }

            }
        }
        ret
    })
}