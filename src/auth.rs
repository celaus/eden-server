extern crate jsonwebtoken as jwt;
extern crate iron;

use std::default::Default;
use self::jwt::{decode, Validation};
use self::jwt::errors::Error;
use error::{StringError, AuthenticationError};
use self::iron::prelude::*;
use self::iron::status;
use self::iron::typemap;
use std::sync::Arc;
use self::iron::Handler;
use self::iron::middleware::{Chain, BeforeMiddleware};
use self::iron::headers::{Bearer, Authorization};
use config::ACLConf;

#[derive(PartialEq, Default, Serialize, Deserialize)]
struct Claims {
    iss: String,
    role: String,
}

fn verify(token: &str, secret: &str) -> Result<Claims, Error> {
    match decode::<Claims>(token, secret.as_bytes(), &Validation::default()) {
        Ok(claims) => Ok(claims.claims),
        Err(e) => Err(e),
    }
}


pub struct ACL {
    client_id: String,
    roles: Vec<String>,
}

pub fn acl_from_conf(conf: ACLConf) -> ACL {
    ACL {
        client_id: conf.name,
        roles: conf.roles,
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct AuthenticatedAgent {
    pub name: String,
    pub role: String,
}

impl typemap::Key for AuthenticatedAgent {
    type Value = AuthenticatedAgent;
}

trait JWTAuthenticator {
    fn authenticate(&self, token: &str) -> Result<AuthenticatedAgent, AuthenticationError>;
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

    fn acl_ok(&self, claims: &Claims) -> bool {
        self.acls
            .iter()
            .filter(|acl| acl.client_id == claims.iss)
            .filter(|acl| acl.roles.contains(&claims.role))
            .count() > 0
    }

    ///
    /// Adds the middleware (self) before the endpoint passed into the function.
    /// returns a usable chain of handlers
    /// * endpoint: H A handler instance that is run after this middleware
    ///
    pub fn add_before<H>(self, endpoint: H) -> Chain
        where H: Handler
    {
        let mut chain = Chain::new(endpoint);
        chain.link_before(self);
        chain
    }
}

impl BeforeMiddleware for JWTAuthenticationMiddleware {
    fn before(&self, req: &mut Request) -> IronResult<()> {
        debug!("Authenticating request: {:?}", req);
        if let Some(bearer) = req.headers.get::<Authorization<Bearer>>() {
            match self.authenticate(&bearer.token) {
                Ok(agent) => {
                    req.extensions.insert::<AuthenticatedAgent>(agent);
                    Ok(())
                }
                _ => Err(IronError::new(AuthenticationError {}, status::Unauthorized)),
            }
        } else {
            Err(IronError::new(AuthenticationError {}, status::Unauthorized))
        }
    }

    fn catch(&self, _: &mut Request, _: IronError) -> IronResult<()> {
        warn!("Error caught in previous middleware");
        Err(IronError::new(StringError { description: "error".to_owned() },
                           status::BadRequest))
    }
}

impl JWTAuthenticator for JWTAuthenticationMiddleware {
    fn authenticate(&self, token: &str) -> Result<AuthenticatedAgent, AuthenticationError> {
        match verify(token, &self.secret) {
            Ok(ref claims) if self.acl_ok(&claims) => {
                debug!("issuer:{}, roles: {}", claims.iss, claims.role);
                Ok(AuthenticatedAgent {
                       name: claims.iss.clone(),
                       role: claims.role.clone(),
                   })
            }
            Ok(_) | Err(_) => Err(AuthenticationError {}),
        }



    }
}
