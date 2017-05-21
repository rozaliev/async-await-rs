use std::rc::Rc;
use std::cell::RefCell;
use std::ops::{State, Generator};



thread_local!(static THREAD_CONTEXT: RefCell<Option<Context>> =  RefCell::new(None) );

pub trait Async<R> {
    fn poll(&mut self) -> Await<R>;
}

pub enum Await<T> {
    Done(T),
    NotReady,
}


pub struct Core {
    new_tasks: Rc<RefCell<Vec<Box<Async<()>>>>>,

    tasks: Vec<Box<Async<()>>>,
}

#[derive(Clone)]
pub struct Context {
    new_tasks: Rc<RefCell<Vec<Box<Async<()>>>>>,
}


// TODO: replace with for<'a> Generator<&'mut Core> when it's available
pub fn context() -> Context {
    THREAD_CONTEXT.with(|c| {
                            c.borrow()
                                .as_ref()
                                .expect("there is no core in this thread")
                                .clone()
                        })
}

impl Core {
    pub fn new() -> Core {
        let new_tasks = Rc::new(RefCell::new(Vec::new()));
        THREAD_CONTEXT.with(|c| {
                                let mut b = c.borrow_mut();
                                *b = Some(Context { new_tasks: new_tasks.clone() });
                            });

        Core {
            new_tasks: new_tasks,
            tasks: Vec::new(),
        }
    }

    pub fn run<T, G: Async<T>>(&mut self, mut g: G) -> T {
        loop {
            if let Await::Done(r) = g.poll() {
                return r;
            }

            // println!("main task not ready");

            let mut compl = Vec::new();
            for (i, g) in self.tasks.iter_mut().enumerate() {
                match g.poll() {
                    Await::NotReady => continue,
                    Await::Done(()) => {
                        compl.push(i);
                    }
                }
            }

            for i in compl {
                println!("task {} completed", i);
                self.tasks.remove(i);
            }


            THREAD_CONTEXT.with(|c| {
                let mut b = c.borrow_mut();
                let new_tasks_ref = &mut b.as_mut().unwrap().new_tasks;
                let mut new_tasks = new_tasks_ref.borrow_mut();

                for t in new_tasks.drain(..) {
                    self.tasks.push(t);
                }

            });


        }


    }
}

impl Context {
    pub fn spawn<G: Async<()> + 'static>(&self, g: G) {
        let mut tasks = self.new_tasks.borrow_mut();
        tasks.push(Box::new(g));
    }
}


impl<R, T> Async<R> for T
    where T: Generator<Return = R, Yield = ()>
{
    fn poll(&mut self) -> Await<R> {
        self.resume(()).into()
    }
}

impl<R> From<State<(), R>> for Await<R> {
    fn from(f: State<(), R>) -> Await<R> {
        match f {
            State::Yielded(()) => Await::NotReady,
            State::Complete(r) => Await::Done(r),
        }
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