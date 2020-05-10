# Mineroute

A Minecraft reverse proxy written in Rust

## Setup
- Clone the [mineroute frontend](https://github.com/aki-ks/mineroute-frontend) and compile it with `ng build --prod`.
- Copy the compiled frontend from the `dist` directory into the `static` directory of the mineroute backend.
- Launch mineroute with `cargo run --release`

Mineroute will accept minecraft connections on port 25565 and can reverse proxy servers running in offline mode.  
The administration frontend is reachable at http://localhost:8080.  
