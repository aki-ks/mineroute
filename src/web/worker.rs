// use actix_web::web;
// use std::sync::{Arc, RwLock};
// use crate::server_state::{Configuration, ServerConfig};

// pub async fn add_server (config: web::Data<Arc<RwLock<Configuration>>>, host : String, server :ServerConfig) {
//     config.write().unwrap().add_server(host.as_ref(), server)
// }

// pub async fn get_servers_worker(config: &web::Data<Arc<RwLock<Configuration>>>) -> &'static str {
//
//     println!("{:?}", config.read().unwrap().get_servers());
//     "me"
// }