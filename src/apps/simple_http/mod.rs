use std::io::{self, Write};
use std::fmt;
use std::ops::{State, Generator};

use core::{Core, context};
use base::Async;
use net::{Conn, TcpListener};

use httparse;


struct Request {
    method: Slice,
    path: Slice,
    version: u8,
    headers: Vec<(Slice, Slice)>,
    data: [u8; 16384],
}

type Slice = (usize, usize);


pub(crate) fn serve() -> impl Async<io::Result<()>> {
    let addr = "127.0.0.1:8080".parse().unwrap();
    let listener = TcpListener::bind(&addr)?;

    loop {
        let conn = await!(listener.accept())?;
        context().spawn(handle_conn(conn));
    }
}


fn handle_conn(mut conn: Conn) -> impl Async<()> {

    let req = match await!(decode(&mut conn)) {
        Ok(r) => r,
        Err(err) => {
            error!("error: {:?}", err);
            return;
        }
    };

    info!("method: {:?}, path: {:?}", req.method(), req.path());
    let mut n = "someone";
    if req.path().starts_with("/hello/") {
        n = &req.path()[7..];
    }


    let mut buf = Vec::new();
    write!(buf,
           "\
        HTTP/1.1 {}\r\n\
        Server: Example\r\n\
        Content-Length: {}\r\n\
        Date: Wed, 24 May 2017 0:00:00 GMT\r\n\
    ",
           200,
           n.len())
            .unwrap();

    buf.write_all("\r\n".as_bytes());
    buf.write_all(n.as_bytes());
    await!(conn.write_all(&buf[..]));
}


fn decode(conn: &mut Conn) -> impl Async<io::Result<Request>> {
    let mut buf = [0; 16384];
    let mut end = 0;


    let req;

    loop {
        let n = await!(conn.read(&mut buf[end..]))?;
        if n == 0 {
            return Err(io::Error::new(io::ErrorKind::UnexpectedEof, "eof"));
        }
        end += n;

        {
            let mut headers = [httparse::EMPTY_HEADER; 16];
            let mut r = httparse::Request::new(&mut headers);

            let status = r.parse(&buf)
                .map_err(|e| {
                             let msg = format!("failed to parse http request: {:?}", e);
                             io::Error::new(io::ErrorKind::Other, msg)
                         })?;
            let amt = match status {
                httparse::Status::Complete(amt) => amt,
                httparse::Status::Partial => continue,
            };

            let toslice = |a: &[u8]| {
                let start = a.as_ptr() as usize - buf.as_ptr() as usize;
                assert!(start < buf.len());
                (start, start + a.len())
            };


            req = Request {
                method: toslice(r.method.unwrap().as_bytes()),
                path: toslice(r.path.unwrap().as_bytes()),
                version: r.version.unwrap(),
                headers: r.headers
                    .iter()
                    .map(|h| (toslice(h.name.as_bytes()), toslice(h.value)))
                    .collect(),
                data: buf,
            };
            break;
        };
    }

    Ok(req)
}

impl Request {
    pub fn method(&self) -> &str {
        ::std::str::from_utf8(self.slice(&self.method)).unwrap()
    }

    fn slice(&self, slice: &Slice) -> &[u8] {
        &self.data[slice.0..slice.1]
    }

    pub fn path(&self) -> &str {
        ::std::str::from_utf8(self.slice(&self.path)).unwrap()
    }
}



impl fmt::Debug for Request {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("Request")
            .field("method", &self.method)
            .field("path", &self.path)
            .field("version", &self.version)
            .field("headers", &self.headers)
            .field("data", &&self.data[..])
            .finish()
    }
}