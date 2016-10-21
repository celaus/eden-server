extern crate iron;

use self::iron::prelude::*;
use self::iron::status;
use self::iron::{Handler};

pub trait RouteProvider {
    fn get_route(&self) -> &str;
}

pub struct TemperaturePressureHandler {
     route: String
}


impl TemperaturePressureHandler {
    pub fn new(route: &str) -> TemperaturePressureHandler {
        TemperaturePressureHandler {
            route: route.to_owned()
        }
    }
}

impl RouteProvider for TemperaturePressureHandler {
    fn get_route(&self) -> &str {
        &self.route
    }
}


impl Handler for TemperaturePressureHandler {
    fn handle(&self, req: &mut Request) -> IronResult<Response> {
        info!("{:?}", req);
        Ok(Response::with((status::Ok, "Hello again !")))
    }
}
