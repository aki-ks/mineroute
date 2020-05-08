use std::sync::{RwLock, Arc};
use crate::server_state::{Configuration, ServerConfig};

use serde::{Serialize, Deserialize};

use actix_web::{get, post, delete, web, App, HttpServer, Responder, HttpResponse};
use std::net::SocketAddr;

//RICHARD: Überhaupt sinnvoll die Funktionen in Woker auszulagern? funktionen hier ja nicht alzu groß?
// use crate::web::worker::{add_server, get_servers_worker};

#[derive(Serialize, Deserialize, Debug)]
struct UploadedServer {
    domain: String,
    sockaddr: String,
}

pub async fn webserver_run(config: Arc<RwLock<Configuration>>) -> std::io::Result<()> {
    HttpServer::new(move || App::new().app_data(config.clone())
        .service(get_servers)
        .service(get_server)
        .service(post_server)
        .service(delete_server)
        )
        .bind("127.0.0.1:8080")?
        .run()
        .await
}

#[post("/servers")]
async fn post_server(_info: web::Path<()>, config: web::Data<Arc<RwLock<Configuration>>>, body: web::Json<UploadedServer>) -> impl Responder {
    match body.sockaddr.parse::<SocketAddr>() {
        Ok(server_address) => {// server hinzufügen
            //RICHARD: ist as_ref hier das richtige? oder wie kann ich den string clonen? .copy geht auf string ja nicht
            config.write().unwrap().add_server(body.domain.as_ref(), ServerConfig::new(server_address));
            HttpResponse::Created().body("Server created")


            body.domain.as_ref()
            &body.domain
        }
        Err(_error) => {HttpResponse::BadRequest().body("Bad Socket Address")}
    }
}



#[get("/api/servers")]
async fn get_servers(_info: web::Path<()>, config: web::Data<Arc<RwLock<Configuration>>>) -> impl Responder {
    // RICHARD: Wie mache ich das zu einem String bzw zu json? Der Reader kann da so nicht ausliefern
    config.read().unwrap().get_servers();
    ""
}

#[delete("/api/servers")]
async fn delete_server(_info: web::Path<()>, config: web::Data<Arc<RwLock<Configuration>>>) -> impl Responder {
    // RICHARD: Wie komm ich an die parameter? https://stackoverflow.com/questions/51156656/how-do-i-get-the-parameters-from-a-post-request-from-a-html-form-in-actix-web
    // ist das der richtige Weg? was mach ich da?
    config.write().unwrap().remove_server("TODO HIER DEN PARAMETER");
    HttpResponse::Ok()
}


/**
{ config.write().unwrap().remove_server("b.mc.de");

match "127.0.0.1:8081".parse() {
Ok(server_address) => {// server hinzufügen
config.write().unwrap().add_server("b.mc.de", ServerConfig::new(server_address))
}
Err(error) => {
error.to_string();
}

};
{
// geschweifte klammern damit kein deadlock
match config.write().unwrap().get_server_mut("b.mc.de") {
//upstream addr wo hin geroutet wird
Some(server) => { server.upstream = "1.1.1.1".parse().unwrap() }
None => {}
}
}

HttpResponse::BadRequest().body("da lief was schief!")}
**/


#[get("/api/server")]
async fn get_server(_info: web::Path<()>, config: web::Data<Arc<RwLock<Configuration>>>) -> impl Responder {
    // RICHARD: Wie komm ich an die parameter? https://stackoverflow.com/questions/51156656/how-do-i-get-the-parameters-from-a-post-request-from-a-html-form-in-actix-web
    // ist das der richtige Weg? was mach ich da?
    match config.read().unwrap().get_server("TODO Hier dem Paramter") {
        Some(_server) => {
            // return server data
            // RICHARD Wie bekomm ich das Server objekt als JSON?
            HttpResponse::Ok()
            }
        None => {HttpResponse::NotFound()}
    }


}

