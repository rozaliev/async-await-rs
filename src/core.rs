use std::os::unix::io::{AsRawFd, RawFd};

use std::collections::HashMap;
use std::rc::Rc;
use std::cell::{Cell, RefCell};

use mio;


use base::{Async, Await};

thread_local!(static THREAD_CONTEXT: RefCell<Option<Context>> =  RefCell::new(None) );

const MAIN_TASK: mio::Token = mio::Token(1);


pub struct Core {
    new_tasks: Rc<RefCell<Vec<Box<Async<()>>>>>,
    tasks: HashMap<mio::Token, Box<Async<()>>>,

    poll: mio::Poll,
    events: mio::Events,
    new_async_interests: Rc<Cell<Option<(RawFd, mio::Ready)>>>,

    last_task_id: usize,
    awaiting: HashMap<mio::Token, Box<Async<()>>>,
}

#[derive(Clone)]
pub struct Context {
    new_tasks: Rc<RefCell<Vec<Box<Async<()>>>>>,
    new_async_interests: Rc<Cell<Option<(RawFd, mio::Ready)>>>,
    current_token: Rc<Cell<mio::Token>>,
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
        let new_async_interests = Rc::new(Cell::new(None));

        THREAD_CONTEXT.with(|c| {
                                let mut b = c.borrow_mut();
                                *b = Some(Context {
                                              new_tasks: new_tasks.clone(),
                                              new_async_interests: new_async_interests.clone(),
                                              current_token: Rc::new(Cell::new(MAIN_TASK)),
                                          });
                            });

        Core {
            new_tasks: new_tasks,
            tasks: HashMap::new(),
            poll: mio::Poll::new().unwrap(),
            events: mio::Events::with_capacity(1024),
            new_async_interests: new_async_interests,
            last_task_id: 1,
            awaiting: HashMap::new(),
        }
    }

    pub fn run<T, G: Async<T>>(&mut self, mut g: G) -> T {
        let mut compl = Vec::new();
        let mut new_awaiting = Vec::new();

        'main: loop {
            set_current_token(MAIN_TASK);
            if let Await::Done(r) = g.poll() {
                return r;
            }

            if let Some(v) = self.new_async_interests.get().take() {
                self.poll
                    .register(&mio::unix::EventedFd(&v.0),
                              MAIN_TASK,
                              v.1,
                              mio::PollOpt::edge())
                    .unwrap();
            }

            println!("main task not ready");

            loop {

                'inner_tasks: for (tok, task) in self.tasks.iter_mut() {
                    set_current_token(*tok);
                    match task.poll() {
                        Await::NotReady => {
                            if let Some(v) = self.new_async_interests.get().take() {
                                self.poll
                                    .register(&mio::unix::EventedFd(&v.0),
                                              *tok,
                                              v.1,
                                              mio::PollOpt::edge())
                                    .unwrap();
                            }
                            new_awaiting.push(*tok);
                        }
                        Await::Done(()) => {
                            compl.push(*tok);
                        }
                    }
                }

                for i in compl.drain(..) {
                    println!("task {:?} completed", i);
                    self.tasks.remove(&i);
                }

                for tok in new_awaiting.drain(..) {
                    println!("task {:?} scheduled to await", tok);
                    if let Some(task) = self.tasks.remove(&tok) {
                        self.awaiting.insert(tok, task);
                    }

                }


                THREAD_CONTEXT.with(|c| {
                    let mut b = c.borrow_mut();
                    let new_tasks_ref = &mut b.as_mut().unwrap().new_tasks;
                    let mut new_tasks = new_tasks_ref.borrow_mut();

                    for t in new_tasks.drain(..) {
                        let next_tok = self.next_tok();
                        self.tasks.insert(next_tok, t);
                    }

                });


                // main fd
                {
                    let mut main_fired = false;
                    let dur = Some(::std::time::Duration::from_millis(100));
                    self.poll.poll(&mut self.events, dur).unwrap();
                    for event in &self.events {
                        let tok = event.token();
                        if tok == MAIN_TASK {
                            main_fired = true;
                        } else {
                            if let Some(task) = self.awaiting.remove(&tok) {
                                self.tasks.insert(tok, task);
                            }
                        }
                    }

                    if main_fired {
                        continue 'main;
                    }
                }
            }




        }


    }

    fn next_tok(&mut self) -> mio::Token {
        self.last_task_id += 1;
        mio::Token(self.last_task_id)
    }
}

impl Context {
    pub fn spawn<G: Async<()> + 'static>(&self, g: G) {
        let mut tasks = self.new_tasks.borrow_mut();
        tasks.push(Box::new(g));
    }

    pub fn register_read<T: AsRawFd>(&self, fd: &T) {
        self.new_async_interests
            .set(Some((fd.as_raw_fd(), mio::Ready::readable())));
    }

    pub fn register_write<T: AsRawFd>(&self, fd: &T) {
        self.new_async_interests
            .set(Some((fd.as_raw_fd(), mio::Ready::writable())));
    }

    pub fn register_all(&self) {}

    fn current_token(&self) -> mio::Token {
        self.current_token.get()
    }
}

fn set_current_token(t: mio::Token) {
    THREAD_CONTEXT.with(|c| c.borrow().as_ref().unwrap().current_token.set(t))
}