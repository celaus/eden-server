extern crate iron;

use self::iron::prelude::*;
use self::iron::status;
use self::iron::Handler;
use self::iron::Iron;
use std::collections::HashMap;
use std::net::ToSocketAddrs;
use std::fmt::Debug;



pub trait RouteProvider {
    fn get_route(&self) -> &str;
}



///
/// A simple request router for (iron)[https://iron.rs],
/// based on a HashMap.
///
pub struct Router {
    // Routes here are simply matched with the url path.
    routes: HashMap<String, Box<Handler>>,
}

impl Router {
    pub fn new() -> Self {
        Router { routes: HashMap::new() }
    }

    pub fn add_route<H>(&mut self, route: String, handler: H)
        where H: Handler
    {
        self.routes.insert(route, Box::new(handler));
    }
}

impl Handler for Router {
    fn handle(&self, req: &mut Request) -> IronResult<Response> {
        let path = req.url.path().join("/");
        match self.routes.get(&path) {
            Some(handler) => handler.handle(req),
            None => Ok(Response::with(status::NotFound)),
        }
    }
}


///
/// An interface for running a web server.
///
pub trait WebServer {
    ///
    /// Start listening on the provided address. Should be Blocking.
    ///
    fn listen<A: ToSocketAddrs + Debug>(self, listen_to: A);
}

///
/// A wrapper for iron's WebServer.
pub struct EdenServer {
    router: Router,
}


impl EdenServer {
    pub fn new(router: Router) -> EdenServer {
        EdenServer { router: router }
    }
}

impl WebServer for EdenServer {
    fn listen<A: ToSocketAddrs + Debug>(self, listen_to: A) {
        info!("Listening to {:?}", listen_to);
        Iron::new(self.router).http(listen_to).unwrap();
    }
}
