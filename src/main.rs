mod net;
mod web;
mod server_state;

use std::env;
use std::net::{SocketAddr, IpAddr, Ipv4Addr};
use std::sync::{RwLock, Arc};
use actix::Actor;
use tokio::net::TcpListener;
use tokio::stream::StreamExt;
use futures_util::future::FutureExt;
use crate::net::manager::ProxyClientManager;
use crate::server_state::{Configuration, ServerConfig};

#[actix_rt::main]
async fn main() {
    let args: Vec<String> = env::args().collect();

    let config = {
        let mut config = Configuration::new();
        config.add_server("a.mc.local", ServerConfig::new("127.0.0.1:25566".parse().unwrap()));
        config.add_server("b.mc.local", ServerConfig::new("127.0.0.1:25567".parse().unwrap()));
        Arc::new(RwLock::new(config))
    };

    actix::spawn(web::webserver_run(config.clone()).map(|_| {}));

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
            ProxyClientManager::new(config.clone(), stream, ctx)
        });
    }
}
