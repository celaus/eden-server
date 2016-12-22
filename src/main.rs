#![feature(proc_macro)]

#[macro_use]
extern crate log;
extern crate log4rs;
extern crate toml;
extern crate cratedb;

#[macro_use]
extern crate serde_derive;


use std::io;
use std::io::Read;
use std::fs::File;
use toml::{Parser, Value};
use std::any::Any;
use std::net::SocketAddr;
use std::sync::mpsc::channel;

mod handler;
mod server;
mod error;
mod consumer;
mod dto;

use server::{RouteProvider, EdenConfig, EdenServer, Router, WebServer};
use handler::{TemperaturePressureHandler, JWTAuthenticationMiddleware};
use dto::TemperaturePressureReading;
use consumer::{CrateDBSink, TemperaturePressureDataSink};
use cratedb::Cluster;
use std::thread;

fn read_config<T: Read + Sized>(mut f: T) -> Result<EdenConfig, io::Error> {
    let mut buffer = String::new();
    try!(f.read_to_string(&mut buffer));
    let root: Value = buffer.parse().unwrap();
    let secret = root.lookup("keys.secret")
        .unwrap_or(&Value::String("asdf".to_owned()))
        .as_str()
        .unwrap()
        .to_owned();
    let port =
        root.lookup("settings.port").unwrap_or(&Value::Integer(6200)).as_integer().unwrap() as u16;

    let raw_addr = root.lookup("settings.listen_address")
        .unwrap_or(&Value::String("0.0.0.0".to_owned()))
        .as_str()
        .unwrap()
        .to_owned();
    let ip: SocketAddr = format!("{}:{}", raw_addr, port).parse().unwrap();

    let cratedb_url = root.lookup("settings.cratedb_url")
        .unwrap_or(&Value::String("localhost:4200".to_owned()))
        .as_str()
        .unwrap()
        .to_owned();

    return Ok(EdenConfig {
        listen_address: ip,
        secret: secret,
        cratedb_url: cratedb_url.to_owned(),
    });
}


fn main() {
    let logging_filename = "logging.yml";
    log4rs::init_file(logging_filename, Default::default()).unwrap();
    info!("Loading configuration");

    let mut f = File::open("./config.toml").unwrap();
    let config = read_config(&mut f).unwrap();


    let (tx, rx) = channel::<TemperaturePressureReading>();

    info!("Starting Eden Server");
    let mut router = Router::new();
    let tp = TemperaturePressureHandler::new("temperature", tx);
    let mw = JWTAuthenticationMiddleware::new("secret".to_owned(), vec![]);
    let temperature_route = tp.get_route().to_owned();
    let handlers = mw.add_before(tp);

    router.add_route(temperature_route, handlers);
    let cratedb_url = config.cratedb_url.clone();
    let consumer = TemperaturePressureDataSink::new(1000);
    thread::spawn(move || {
        let mut c: Cluster = Cluster::from_string(cratedb_url).unwrap();
        consumer.relay(rx, c);
    });

    let srv = EdenServer::new(config, router);
    srv.listen();
}
