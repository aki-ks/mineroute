mod net;

use std::net::{SocketAddr, IpAddr, Ipv4Addr};
use std::env;
use actix::Actor;
use tokio::net::TcpListener;
use tokio::stream::StreamExt;
use crate::net::manager::ProxyClientManager;

#[actix_rt::main]
async fn main() {
    let args: Vec<String> = env::args().collect();

    let port: u16 = args.get(1).map(|arg| arg.parse().unwrap())
        .unwrap_or(25565);

    let addr = {
        let localhost = IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1));
        SocketAddr::new(localhost, port)
    };

    let mut listener = TcpListener::bind(&addr).await.unwrap();
    let mut connections = listener.incoming();
    while let Some(Ok(stream)) = connections.next().await {
        ProxyClientManager::create(|ctx| {
            ProxyClientManager::new(stream, ctx)
        });
    }
}
