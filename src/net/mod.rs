use std::net::SocketAddr;
use std::io::{self, Read, Write};

use std::ops::{Generator, State};

use mio::tcp::TcpListener as MioTcpListener;
use mio::net::TcpStream;

macro_rules! nb_yield {
    ($fd:expr, $e:expr, Read) => (
        nb_yield!($e, {
            ::core::context().register_read(&$fd);
        })
    );

    ($fd:expr, $e:expr, Write) => (
        nb_yield!($e, {
            ::core::context().register_read(&$fd);
        })
    );

    ($fd:expr, $e:expr, All) => (
        nb_yield!($e, {
            ::core::context().register_all(&$fd);
        })
    );

    ($e:expr, $i:expr) => ({
        let ret;
        loop {
            match $e {
                Ok(v) => { ret = Ok(v); break },
                Err(ref err) if err.kind() == io::ErrorKind::WouldBlock => {
                    $i
                    yield
                },
                Err(err) => { ret = Err(err); break },
            }
        }
        ret
    })
}


pub struct TcpListener(MioTcpListener);
pub struct Conn(TcpStream);

impl TcpListener {
    pub fn bind(addr: &SocketAddr) -> io::Result<TcpListener> {
        let listener = MioTcpListener::bind(addr)?;

        Ok(TcpListener(listener))
    }


    pub fn accept(&self) -> impl Generator<Return = io::Result<Conn>, Yield = ()> {
        let (stream, _) = nb_yield!(self.0, self.0.accept(), Read)?;
        Ok(Conn(stream))
    }
}

impl Conn {
    pub fn read(&mut self,
                buf: &mut [u8])
                -> impl Generator<Return = io::Result<usize>, Yield = ()> {

        nb_yield!(self.0, self.0.read(buf), Read)
    }

    pub fn write_all(&mut self,
                     mut buf: &[u8])
                     -> impl Generator<Return = io::Result<()>, Yield = ()> {

        while !buf.is_empty() {
            match await!(self.write(buf)) {
                Ok(0) => {
                    return Err(io::Error::new(io::ErrorKind::WriteZero,
                                              "failed to write whole buffer"))
                }
                Ok(n) => buf = &buf[n..],
                Err(ref e) if e.kind() == io::ErrorKind::Interrupted => {}
                Err(e) => return Err(e),
            }
        }
        Ok(())
    }

    pub fn write(&mut self, buf: &[u8]) -> impl Generator<Return = io::Result<usize>, Yield = ()> {
        nb_yield!(self.0, self.0.write(buf), Write)
    }
}
