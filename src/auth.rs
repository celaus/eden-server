extern crate simple_jwt;
extern crate iron;

use error::{StringError, AuthenticationError};
use self::iron::prelude::*;
use self::iron::status;
use self::iron::typemap;
use self::simple_jwt::decode;
use std::sync::Arc;
use self::iron::Handler;
use self::iron::middleware::{Chain, BeforeMiddleware};
use self::iron::headers::{Bearer, Authorization};
use config::ACLConf;

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

#[derive(Serialize, Debug)]
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
                },
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
        let claim = decode(token, &self.secret).map_err(|_| AuthenticationError {})?;
        let role = claim.payload
            .get("role")
            .ok_or(AuthenticationError {})?
            .as_str()
            .ok_or(AuthenticationError {})?
            .to_owned();
        let issuer_json = claim.registered.iss.ok_or(AuthenticationError {})?;
        let issuer = issuer_json.as_str();

        debug!("issuer:{}, roles: {}", issuer, role);
        let v = self.acls
            .iter()
            .filter(|acl| acl.client_id == issuer)
            .filter(|acl| acl.roles.contains(&role));
        if v.count() > 0 {
            Ok(AuthenticatedAgent {
                name: issuer.to_owned(),
                role: role.to_string(),
            })
        } else {
            debug!("Access denied for issuer: {} ({})", issuer, role);
            Err(AuthenticationError {})
        }
    }
}
