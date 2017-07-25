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

extern crate toml;
extern crate serde;

use error::ConfigError;
use std::io::Read;

#[derive(Deserialize)]
pub struct Settings {
    pub keys: Keys,
    pub http: Http,
    pub mqtt: MQTT,
    pub cratedb: CrateDb,
    pub acls: Vec<ACLConf>,
}

#[derive(Deserialize)]
pub struct Keys {
    pub secret: String,
}

#[derive(Deserialize)]
pub struct Http {
    pub enable: bool,
    pub listen_address: String,
}
#[derive(Deserialize)]
pub struct MQTT {
    pub broker_address: String,
    pub username: String,
    pub password: String,
    pub verify_ca: bool,
    pub topics: Vec<String>,
}


#[derive(Deserialize)]
pub struct CrateDb {
    pub url: String,
    pub bulk_size: usize,
    pub create_statement: String,
    pub insert_statement: String,
}

#[derive(Deserialize)]
pub struct ACLConf {
    pub name: String,
    pub roles: Vec<String>,
}

pub fn read_config<T: Read + Sized>(mut f: T) -> Result<Settings, ConfigError> {
    let mut buffer = String::new();
    try!(f.read_to_string(&mut buffer).map_err(ConfigError::Io));
    toml::from_str(&buffer).map_err(ConfigError::Parse)
}
