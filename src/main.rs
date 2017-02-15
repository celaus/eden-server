#[macro_use]
extern crate log;
extern crate log4rs;
extern crate cratedb;

#[macro_use]
extern crate serde_derive;
extern crate clap;

use std::fs::File;
use std::sync::mpsc::channel;

mod handler;
mod server;
mod error;
mod consumer;
mod auth;
mod config;
mod datasink;

use config::read_config;
use auth::{AuthenticatedAgent, acl_from_conf, JWTAuthenticationMiddleware};
use server::{RouteProvider, EdenServer, Router, WebServer};
use handler::{Message, TemperaturePressureHandler};
use consumer::SensorDataSink;
use datasink::CrateDBSink;
use cratedb::Cluster;
use std::thread;
use clap::{Arg, App};
use std::sync::Arc;


fn main() {
    let matches = App::new("Eden Server")
        .version("0.2.0")
        .author("Claus Matzinger. <claus.matzinger+kb@gmail.com>")
        .about("Receives Eden Client data, authenticates via JWT, and pushes it to a CrateDB \
                cluster")
        .arg(Arg::with_name("config")
            .short("c")
            .long("config")
            .help("Sets a custom config file [default: config.toml]")
            .value_name("config.toml")
            .takes_value(true))
        .arg(Arg::with_name("logging")
            .short("l")
            .long("logging-conf")
            .value_name("logging.yml")
            .takes_value(true)
            .help("Sets the logging configuration [default: logging.yml]"))
        .get_matches();

    let config_filename = matches.value_of("config").unwrap_or("config.toml");
    let logging_filename = matches.value_of("logging").unwrap_or("logging.yml");
    info!("Using configuration file '{}' and logging config '{}'",
          config_filename,
          logging_filename);

    log4rs::init_file(logging_filename, Default::default()).unwrap();
    let mut f = File::open(config_filename).unwrap();
    let settings = read_config(&mut f).unwrap();


    let (tx, rx) = channel::<(Arc<AuthenticatedAgent>, Message)>();

    info!("Starting Eden Server");

    let mut router = Router::new();
    let tp = TemperaturePressureHandler::new("temperature", tx);


    let acls = settings.acls
        .into_iter()
        .map(|acl| acl_from_conf(acl))
        .collect();

    let mw = JWTAuthenticationMiddleware::new(settings.keys.secret.clone(), acls);
    let temperature_route = tp.get_route().to_owned();
    let handlers = mw.add_before(tp);

    router.add_route(temperature_route, handlers);
    let cratedb_url = settings.cratedb.url.clone();
    let consumer = SensorDataSink::new();
    let bulk_size = settings.cratedb.bulk_size;

    let insert_thread = thread::spawn(move || {
        let c: Cluster = Cluster::from_string(cratedb_url).unwrap();
        consumer.relay(rx, c, bulk_size);
    });

    let srv = EdenServer::new(router);

    // call blocking handler
    let addr: &str = &settings.http.listen_address;
    srv.listen(addr);

    // insert all data from the queue
    let _ = insert_thread.join();
}
