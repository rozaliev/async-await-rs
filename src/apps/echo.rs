use std::io;
use std::ops::{State, Generator};

use core::{Core, context};
use base::Async;
use net::{Conn, TcpListener};

pub(crate) fn serve() -> impl Async<io::Result<()>> {
    let addr = "127.0.0.1:12345".parse().unwrap();
    let listener = TcpListener::bind(&addr)?;

    loop {
        let conn = await!(listener.accept())?;
        context().spawn(handle_conn(conn));
    }
}

fn handle_conn(mut conn: Conn) -> impl Async<()> {
    let mut buf = [0; 1024];

    loop {
        let n = await!(conn.read(&mut buf)).unwrap();
        if n == 0 {
            debug!("eof");
            return;
        }
        trace!("read {} bytes: {:?}", n, &buf[0..n]);
        await!(conn.write_all(&buf[0..n]));
        trace!("buf written");
    }
}