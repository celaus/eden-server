extern crate iron;

use std::net::SocketAddr;

use self::iron::prelude::*;
use self::iron::status;
use self::iron::{Handler};
use std::collections::HashMap;


pub trait RouteProvider {
    fn get_route(&self) -> &str;
}


pub struct Router {
    // Routes here are simply matched with the url path.
    routes: HashMap<String, Box<Handler>>
}

impl Router {
    pub fn new() -> Self {
        Router { routes: HashMap::new() }
    }

    pub fn add_route<H>(&mut self, route: String, handler: H) where H: Handler {
        self.routes.insert(route, Box::new(handler));
    }
}

impl Handler for Router {
    fn handle(&self, req: &mut Request) -> IronResult<Response> {
        let path = req.url.path().join("/");
        match self.routes.get(&path) {
            Some(handler) => handler.handle(req),
            None => Ok(Response::with(status::NotFound))
        }
    }
}

pub trait WebServer {
    fn listen(self);
}

pub struct EdenServer {
    listen_address: SocketAddr,
    secret: String,
    router: Router,
}

pub struct EdenConfig {
    pub listen_address: SocketAddr,
    pub secret: String,
    pub cratedb_url: String
}


impl EdenServer {
    pub fn new(config: EdenConfig, router: Router) -> EdenServer {
        EdenServer {
            listen_address: config.listen_address,
            secret: config.secret,
            router: router,
        }
    }
}

impl WebServer for EdenServer {
    fn listen(self) {
        info!("Listening to {}", self.listen_address);
        Iron::new(self.router).http(self.listen_address).unwrap();
    }
}
