extern crate iron;
extern crate bodyparser;
extern crate persistent;
extern crate simple_jwt;


use error::{StringError, AuthenticationError};
use self::iron::prelude::*;
use self::iron::status;
use self::iron::method::*;
use self::iron::Handler;
use self::iron::middleware::{Chain, BeforeMiddleware};
use self::iron::headers::{Bearer, Authorization};
use std::error::Error;
use std::fmt::{self, Debug};
use server::RouteProvider;
use self::simple_jwt::{decode, Algorithm};
use std::sync::mpsc::Sender;
use std::sync::Mutex;
use dto::TemperaturePressureReading;
use self::persistent::Read;
use std::rc::Rc;
use std::sync::Arc;

trait JWTAuthenticator {
    fn authenticate(&self, token: &String) -> Result<(), AuthenticationError>;
}


pub struct JWTAuthenticationMiddleware {
    acls: Arc<Vec<ACL>>,
    secret: String,
}

impl JWTAuthenticationMiddleware {
    pub fn new(secret: String, acls: Vec<ACL>) -> JWTAuthenticationMiddleware {
        JWTAuthenticationMiddleware {
            acls: Arc::new(acls),
            secret: secret,
        }
    }

    ///
    /// Adds the middleware (self) before the endpoint passed into the function.
    /// returns a usable chain of handlers
    /// * endpoint: H A handler instance that is run after this middleware
    ///
    pub fn add_before<H>(self, endpoint: H) -> Chain where H: Handler {
        let mut chain = Chain::new(endpoint);
        chain.link_before(self);
        chain
    }
}

impl BeforeMiddleware for JWTAuthenticationMiddleware {
    fn before(&self, req: &mut Request) -> IronResult<()> {
        if let Some(bearer) = req.headers.get::<Authorization<Bearer>>() {
            match self.authenticate(&bearer.token) {
                Ok(_) => Ok(()),
                _ => Err(IronError::new(AuthenticationError {}, status::Unauthorized)),
            }
        } else {
            Err(IronError::new(AuthenticationError {}, status::Unauthorized))
        }
    }
    fn catch(&self, _: &mut Request, err: IronError) -> IronResult<()> {
        Err(IronError::new(StringError { description: "error".to_owned() }, status::BadRequest))
    }
}

impl JWTAuthenticator for JWTAuthenticationMiddleware {
    fn authenticate(&self, token: &String) -> Result<(), AuthenticationError> {
        let claim = decode(&token, &self.secret).map_err(|e| AuthenticationError {})?;
        let role = claim.payload
            .get("roles")
            .ok_or(AuthenticationError {})?
            .as_str()
            .ok_or(AuthenticationError {})?
            .to_owned();
        let issuer_json = claim.registered.iss.ok_or(AuthenticationError {})?;
        let issuer = issuer_json.as_str();
        let v = self.acls
            .iter()
            .filter(|acl| acl.client_id == issuer)
            .filter(|acl| acl.roles.contains(&role));
        if v.count() > 0 {
            Ok(())
        } else {
            Err(AuthenticationError {})
        }
    }
}

pub struct ACL {
    client_id: String,
    roles: Vec<String>,
}

pub struct TemperaturePressureHandler {
    route: String,
    sender: Mutex<Sender<TemperaturePressureReading>>,
}


impl TemperaturePressureHandler {
    pub fn new(route: &str,
               sender: Sender<TemperaturePressureReading>)
               -> TemperaturePressureHandler {
        TemperaturePressureHandler {
            route: route.to_owned(),
            sender: Mutex::new(sender),
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
        match req.method {
            Method::Put => {
                let json_body = req.get::<bodyparser::Struct<TemperaturePressureReading>>();
                if let Ok(body) = json_body {
                    if let Some(content) = body {
                        self.sender.lock().unwrap().send(content);
                        return Ok(Response::with((status::Ok, "")));
                    }
                }
                Ok(Response::with((status::BadRequest, "")))
            }
            _ => {
                Err(IronError::new(StringError { description: "Error".to_string() },
                                   status::MethodNotAllowed))
            }

        }

    }
}
