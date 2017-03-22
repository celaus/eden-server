// Copyright 2016 Claus Matzinger
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//    http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

extern crate iron;
extern crate bodyparser;


use error::StringError;
use self::iron::prelude::*;
use self::iron::status;
use self::iron::method::*;
use self::iron::Handler;
use server::RouteProvider;
use std::sync::mpsc::Sender;
use std::sync::{Arc, Mutex};
use auth::AuthenticatedAgent;



#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Message {
    pub meta: MetaData,
    pub data: Vec<Measurement>,
    pub timestamp: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Measurement {
    pub sensor: String,
    pub value: f64,
    pub unit: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MetaData {
    pub name: String,
}


pub struct SensorDataHandler {
    route: String,
    sender: Mutex<Sender<(Arc<AuthenticatedAgent>, Message)>>,
}


impl SensorDataHandler {
    pub fn new(route: &str,
               sender: Sender<(Arc<AuthenticatedAgent>, Message)>)
               -> SensorDataHandler {
        SensorDataHandler {
            route: route.to_owned(),
            sender: Mutex::new(sender),
        }
    }
}

impl RouteProvider for SensorDataHandler {
    fn get_route(&self) -> &str {
        &self.route
    }
}


impl Handler for SensorDataHandler {
    fn handle(&self, req: &mut Request) -> IronResult<Response> {
        match req.method {
            Method::Put | Method::Post => {
                let json_body = req.get::<bodyparser::Struct<Vec<Message>>>();
                debug!("Received: {:?}", json_body);
                match json_body {
                    Ok(Some(content)) => {
                        let agent =
                            Arc::new(req.extensions.remove::<AuthenticatedAgent>().unwrap());
                        info!("Received {} messages from {:?}", content.len(), agent);

                        for msg in content {
                            let _ = self.sender.lock().unwrap().send((agent.clone(), msg));
                        }
                        Ok(Response::with((status::Ok, "[\"Done\"]")))
                    }
                    Ok(None) | Err(_) => {
                        info!("JSON body could not be parsed.");
                        Err(IronError::new(StringError { description: "invalid body".to_string() },
                                           status::BadRequest))
                    }
                }
            }
            _ => {
                info!("Method not allowed");
                Err(IronError::new(StringError { description: "Error".to_string() },
                                   status::MethodNotAllowed))
            }
        }
    }
}
