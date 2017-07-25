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

extern crate rumqtt;

use serde_json;
use handler::Message;
use std::sync::mpsc::Sender;
use std::sync::{Arc, Mutex};
use auth::AuthenticatedAgent;
use self::rumqtt::{Message as MqttMessage, MqttOptions, MqttClient, MqttCallback, QoS};



pub struct MqttSubscription {
    topics: Vec<String>,
    options: MqttOptions,
}


impl MqttSubscription {
    pub fn new<T, U, P, A>(topics: Vec<T>,
                           username: U,
                           password: P,
                           address: A,
                           verify_ca: bool)
                           -> MqttSubscription
        where T: Into<String>,
              U: Into<String>,
              A: Into<String>,
              P: Into<String>
    {
        let options = MqttOptions::new()
            .set_keep_alive(5)
            .set_reconnect(3)
            .set_q_timeout(15)
            .set_user_name(&username.into())
            .set_password(&password.into())
            .set_should_verify_ca(verify_ca)
            .set_client_id("x5ff mqtt")
            .set_broker(&address.into());

        MqttSubscription {
            topics: topics.into_iter().map(|t| t.into()).collect(),
            options: options,
        }
    }

    pub fn start(self, sender: Sender<(Arc<AuthenticatedAgent>, Message)>) {
        let sender = Mutex::new(sender);
        let msg_callback = MqttCallback::new()
            .on_message(move |m: MqttMessage| if let Some(userdata_raw) = m.userdata {
                            let userdata = String::from_utf8_lossy(&userdata_raw);
                            let data = String::from_utf8_lossy(&m.payload);
                            let readings_raw: Result<Vec<Message>, _> = serde_json::from_str(&data);
                            if let Ok(readings) = readings_raw {
                                let agent = Arc::new(AuthenticatedAgent {
                                                         name: userdata.to_string(),
                                                         role: userdata.to_string(),
                                                     });
                                for msg in readings {
                                    let _ = sender.lock().unwrap().send((agent.clone(), msg));
                                }
                            }
                        });
        let mut request = MqttClient::start(self.options, Some(msg_callback)).unwrap();
        let _ = request.subscribe(self.topics
                                      .iter()
                                      .map(|t| (t.as_ref(), QoS::Level1))
                                      .collect());
    }
}
