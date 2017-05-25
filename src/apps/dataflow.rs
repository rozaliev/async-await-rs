// use std::ops::{State, Generator};
// use std::collections::HashMap;
// use std::hash::Hash;

// #[macro_export]
// macro_rules! get {
//     ($g: ident, $e: expr) => ({
//         match $g.resume($e) {
//             State::Yielded(i) => i,
//             State::Complete(_) => unreachable!()
//         }
//     })
// }

// trait Source {
//     type Item;

//     fn pull(&mut self) -> Self::Item;
// }


// trait Operator<T> {
//     type Out;

//     fn process(&mut self, input: T) -> Self::Out;
// }




// fn nums() -> impl Generator<Return = !, Yield = u64> {
//     let mut i = 0;
//     loop {
//         yield i;
//         i += 1;
//     }
// }

// fn n_plus2X(n: u64) -> impl Generator<u64, Return = !, Yield = u64> {
//     loop {
//         let inp = gen arg;

//         yield n + 2 * inp;
//     }
// }

// fn rem(n: u64) -> impl Generator<u64, Return = !, Yield = u64> {
//     loop {
//         let inp = gen arg;

//         yield inp % n;
//     }
// }



// fn memo<T>(mut target: T) -> impl Generator<u64, Return = !, Yield=u64>
// where T: Generator<u64,Return=!,Yield=u64> {
//     let mut hm = HashMap::<u64,u64>::new();

//     loop {
//         let inp = gen arg;
//         if let Some(v) = hm.get(&inp) {
//             yield *v;
//             continue
//         }

//         let r = get!(target, inp);
//         hm.insert(inp, r);
//         yield r
//     }
// }



pub fn run() {
    println!("it breaks llvm lol");
    // let mut source = nums();
    // let mut op1 = n_plus2X(333);
    // let mut op2 = rem(64);
    // let op3 = n_plus2X(666);
    // let mut m = memo(op3);

    // loop {
    //     let i = get!(source, ());
    //     let p1 = get!(op1,i);
    //     let p2 = get!(op2, p1);
    //     let p3 = get!(m, p2);
    // }
}
