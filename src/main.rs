use std::net::{Ipv4Addr, SocketAddr, SocketAddrV4};
use std::str::FromStr;

use axum::serve;
use clap::Parser;
use log::info;
use tokio::net::TcpListener;

mod args;
mod controllers;
mod middleware;
mod models;
mod routes;
mod services;

use crate::args::Args;
use crate::models::state::State;
use crate::routes::router::create_router;

#[cfg(test)]
mod tests;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    env_logger::init_from_env(
        env_logger::Env::default().filter_or(env_logger::DEFAULT_FILTER_ENV, "debug"),
    );

    let args = Args::parse();
    let address = &args.address.split(':').collect::<Vec<&str>>();
    let addr = SocketAddrV4::new(
        Ipv4Addr::from_str(address[0]).unwrap(),
        address[1].parse().unwrap(),
    );

    let state = State::new();
    let app = create_router(state, args);

    info!("Server listening on {}", addr);

    // Create a TCP listener & service
    let listener = TcpListener::bind(addr).await?;
    let service = app.into_make_service_with_connect_info::<SocketAddr>();
    // Serve with both the listener and the service
    serve(listener, service).await?;
    Ok(())
}
