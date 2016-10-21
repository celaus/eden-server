#[macro_use]
extern crate log;
extern crate log4rs;
extern crate toml;


use std::io;
use std::io::Read;
use std::sync::Arc;
use std::fs::File;
use toml::{Parser, Value};
use std::any::Any;
use std::net::SocketAddr;

mod handler;
mod server;
use server::{EdenConfig, EdenServer, Router, WebServer};
use handler::{RouteProvider, TemperaturePressureHandler};



fn read_config<T: Read + Sized>(mut f: T) -> Result<EdenConfig, io::Error> {
    let mut buffer = String::new();
    try!(f.read_to_string(&mut buffer));
    let root: Value = buffer.parse().unwrap();
    let secret = root.lookup("keys.secret").unwrap_or(&Value::String("asdf".to_owned())).as_str().unwrap().to_owned();
    let port = root.lookup("settings.port").unwrap_or(&Value::Integer(6200)).as_integer().unwrap() as u16;

    let raw_addr = root.lookup("settings.listen_address").unwrap_or(&Value::String("0.0.0.0".to_owned())).as_str().unwrap().to_owned();
    let ip:SocketAddr = format!("{}:{}", raw_addr, port).parse().unwrap();
    return Ok(EdenConfig { listen_address: ip, secret: secret});
}


fn main() {
    let logging_filename = "logging.yml";
    log4rs::init_file(logging_filename, Default::default()).unwrap();
    info!("Loading configuration");

    let mut f = File::open("./config.toml").unwrap();
    let config = read_config(&mut f).unwrap();

    info!("Starting Eden");

    let mut router = Router::new();
    let tp = TemperaturePressureHandler::new("temperature");
    router.add_route(tp.get_route().to_owned(), tp);

    //
    //router.add_route("hello/again".to_string(), |_: &mut Request| {
    //   Ok(Response::with((status::Ok, "Hello again !")))
    //});
    //
    //router.add_route("error".to_string(), |_: &mut Request| {
    //   Ok(Response::with(status::BadRequest))
    //});



    let srv = EdenServer::new(config, router);
    srv.listen();
}
