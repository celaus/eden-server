extern crate iron;


use std::net::SocketAddr;

use self::iron::prelude::*;
use self::iron::status;
use self::iron::{Handler};
use handler::{RouteProvider};
use std::collections::HashMap;



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
        match self.routes.get(&req.url.path().join("/")) {
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
    pub secret: String
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
        Iron::new(self.router).http(self.listen_address).unwrap();
    }
}
