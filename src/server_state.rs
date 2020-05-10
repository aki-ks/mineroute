use std::collections::HashMap;
use std::net::SocketAddr;
use std::sync::RwLock;

pub struct Configuration {
    /// Map Hostnames to the corresponding servers
    servers: HashMap<String, ServerConfig>
}

impl Configuration {
    pub fn new() -> Configuration {
        Configuration{
            servers: HashMap::new()
        }
    }

    pub fn get_server_hosts(&self) -> Vec<&String> {
        self.servers.keys().collect()
    }

    pub fn add_server(&mut self, host: &str, server: ServerConfig) {
        self.servers.insert(host.to_owned(), server);
    }

    pub fn remove_server(&mut self, host: &str) -> Option<ServerConfig> {
        self.servers.remove(host)
    }

    pub fn get_server(&self, host: &str) -> Option<&ServerConfig> {
        self.servers.get(host)
    }

    pub fn get_server_mut(&mut self, host: &str) -> Option<&mut ServerConfig> {
        self.servers.get_mut(host)
    }
}

pub struct ServerConfig {
    pub upstream: SocketAddr,
    pub players: RwLock<Vec<String>>,
}

impl ServerConfig {
    pub fn new(upstream: SocketAddr) -> ServerConfig {
        ServerConfig {
            upstream,
            players: RwLock::new(Vec::new()),
        }
    }

    pub fn add_player(&mut self, player: String) {
        self.players.write().unwrap().push(player);
    }

    pub fn remove_player(&mut self, player: &str) {
        let mut players = self.players.write().unwrap();
        if let Some(index) = players.iter().position(|name| name == player) {
            players.remove(index);
        }
    }
}
