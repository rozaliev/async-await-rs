use std::ops::{State, Generator};

struct IterGen<T>(T);


impl<T,I> Iterator for IterGen<T> where T: Generator<Return=(), Yield=I>{
    type Item = I;

    fn next(&mut self) -> Option<Self::Item> {
        match self.0.resume(()) {
            State::Yielded(i) => return Some(i),
            State::Complete(_) => return None
        }
    }
}

pub fn run() {
    for i in inf() {
        println!("n: {}", i);
    }

}

fn inf() -> IterGen<impl Generator<Return=(), Yield=u64>> {
    // wrap for usability since we can't implement Iterator for T: Generator
    fn work() -> impl Generator<Return=(), Yield=u64> {
        let mut i = 0;
        loop {
            yield i;
            i += 1;
        }
    }
    
    IterGen(work())
}