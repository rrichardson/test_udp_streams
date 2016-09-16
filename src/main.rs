
extern crate futures;
extern crate tokio_core;
extern crate tokio_timer;

use std::net::ToSocketAddrs;
use std::str;
use futures::Future;
use futures::stream::Stream;
use tokio_core::reactor::Core;
use tokio_core::net::stream::Udp;
use tokio_core::net::{ UdpSocket, Buffer, BufferPool, VecBuffer, VecBufferPool };
use tokio_timer::Timer;
use std::time::Duration;

fn main() {
    let mut core = Core::new().unwrap();
    let srvaddr = "127.0.0.1:9999".to_socket_addrs().unwrap().next().unwrap();
    let cliaddr = "127.0.0.1:9998".to_socket_addrs().unwrap().next().unwrap();

    let mut clipool = VecBufferPool::new(1024 * 8);
    let mut srvpool = VecBufferPool::new(1024 * 8);

    let mut srv = UdpSocket::bind(&srvaddr, &core.handle()).unwrap();
    let mut cli = UdpSocket::bind(&cliaddr, &core.handle()).unwrap();
   
    let mut srv2 = srv.try_clone(&core.handle()).unwrap();
    let mut cli2 = cli.try_clone(&core.handle()).unwrap();
    let mut cli3 = cli.try_clone(&core.handle()).unwrap();

    let mut srvstream = Udp::new(srv, srvpool);
    let mut clistream = Udp::new(cli, clipool);

    
    let app = cli3.send_all_to(b"PING", &srvaddr).and_then(|_| {
        let server = srvstream.for_each(|(mut buf, addr)| { 
            println!("{}", str::from_utf8(buf.as_mut()).unwrap());
            srv2.send_all_to(b"PONG", &addr).map(|_| ()).wait()
        });
        let client = clistream.for_each(|(mut buf, addr)| { 
            println!("{}", str::from_utf8(buf.as_mut()).unwrap());
            cli2.send_all_to(b"PING", &addr).map(|_| ()).wait()
        });
        server.join(client)
    });

    let timer = Timer::default(); 
    let wait = timer.timeout(app, Duration::from_millis(500));

    core.run(wait).unwrap();
}

