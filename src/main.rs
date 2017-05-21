#![feature(conservative_impl_trait)]
#![feature(generator_trait)]
#![feature(generators)]

extern crate mio;

#[macro_use]
mod base;
mod net;

use std::io;
use std::ops::{State, Generator};

use base::{Core, context, Async};
use net::{Conn, TcpListener};

fn main() {
    let mut core = Core::new();

    core.run(serve());
}

#[inline]
fn serve() -> impl Async<io::Result<()>> {
    let addr = "127.0.0.1:12345".parse().unwrap();
    let listener = TcpListener::bind(&addr)?;

    loop {
        let conn = await!(listener.accept())?;
        context().spawn(handle_conn(conn));
    }

    Ok(())

}

fn handle_conn(mut conn: Conn) -> impl Async<()> {
    let mut buf = [0; 1024];

    loop {
        let n = await!(conn.read(&mut buf)).unwrap();
        if n == 0 {
            println!("eof");
            return;
        }
        println!("read {} bytes: {:?}", n, &buf[0..n]);
        await!(conn.write_all(&buf[0..n]));
        println!("buf written");
    }
}