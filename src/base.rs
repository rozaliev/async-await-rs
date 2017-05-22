use std::ops::{State, Generator};





pub trait Async<R> {
    fn poll(&mut self) -> Await<R>;
}

pub enum Await<T> {
    Done(T),
    NotReady,
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