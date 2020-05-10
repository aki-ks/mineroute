use std::io;
use std::sync::{RwLock, Arc};
use std::net::SocketAddr;
use serde::{Serialize, Deserialize};
use actix_web::*;
use actix_web::body::Body;
use actix_web::web::HttpResponse;
use actix_files::Files;
use crate::server_state::{Configuration, ServerConfig};

type Conf = Arc<RwLock<Configuration>>;

#[derive(Serialize, Deserialize, Debug)]
struct Server {
    domain: String,
    sockaddr: String,
}

#[derive(Serialize, Deserialize, Debug)]
struct ServerStatus {
    domain: String,
    sockaddr: String,
    players: Vec<String>,
}

pub async fn webserver_run(config: Conf) -> io::Result<()> {
    let server = HttpServer::new(move || {
        App::new().data(config.clone())
            .service(get_servers)
            .service(get_server)
            .service(post_server)
            .service(delete_server)
            .service(Files::new("/", "static/").index_file("index.html"))
    });

    server.bind("127.0.0.1:8080")?.run().await
}

#[post("/api/servers")]
async fn post_server(config: web::Data<Conf>, body: web::Json<Server>) -> impl Responder {
    match body.sockaddr.parse::<SocketAddr>() {
        Ok(server_address) => {
            let server = ServerConfig::new(server_address.clone());
            config.write().unwrap().add_server(&body.domain, server);
            HttpResponse::Created().json(Server {
                domain: body.domain.clone(),
                sockaddr: body.sockaddr.clone(),
            })
        }
        Err(_error) => HttpResponse::BadRequest().body("Bad Socket Address"),
    }
}

#[get("/api/servers")]
async fn get_servers(config: web::Data<Conf>) -> impl Responder {
    let config = config.read().unwrap();
    let server_list: Vec<_> = config.get_server_hosts().iter()
        .map(|server_socket| {
            let server_data = config.get_server(server_socket).unwrap();
            Server {
                domain: (*server_socket).clone(),
                sockaddr: server_data.upstream.to_string(),
            }
        })
        .collect();

    HttpResponse::Ok().json(&server_list)
}

#[delete("/api/servers/{key}")]
async fn delete_server(host: web::Path<String>, config: web::Data<Conf>) -> impl Responder {
    let server = config.write().unwrap().remove_server(&host);
    if let Some(server) = server {
        respond_with_server(&host, &server)
    } else {
        HttpResponse::NotFound().finish()
    }
}

#[get("/api/servers/{key}")]
async fn get_server(host: web::Path<String>, config: web::Data<Conf>) -> impl Responder {
    let config = config.read().unwrap();
    if let Some(server) = config.get_server(&host) {
        respond_with_server(&host, server)
    } else {
        HttpResponse::NotFound().finish()
    }
}

fn respond_with_server(domain: &str, server: &ServerConfig) -> HttpResponse<Body> {
    HttpResponse::Ok().json(ServerStatus {
        domain: domain.parse().unwrap(),
        sockaddr: server.upstream.to_string(),
        players: server.players.read().unwrap().clone(),
    })
}
