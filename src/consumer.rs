
extern crate cratedb;
extern crate serde;
extern crate serde_json;
extern crate hex;

use self::cratedb::{Nothing, Cluster};
use self::cratedb::sql::QueryRunner;
use self::cratedb::blob::BlobContainer;
use datasink::CrateDBSink;
use std::sync::mpsc::Receiver;
use handler::{Message, Measurement};
use std::time::Duration;
use std::borrow::Cow;
use auth::AuthenticatedAgent;
use std::sync::Arc;
use std::collections::BTreeMap;
use self::serde_json::Value;
use std::io::Cursor;
use self::hex::ToHex;


#[derive(Serialize)]
struct SensorData {
    value: Value,
    unit: String,
}

#[derive(Serialize)]
struct DeviceData {
    name: String,
    agent: String,
    role: String,
}


pub struct SensorDataSink {
    init_statement: String,
    insert_statement: String,
    table_name: String,
}


impl SensorDataSink {
    pub fn new<C, I, T>(init_statement: C, insert_statement: I, table_name: T) -> SensorDataSink
        where I: Into<String>,
              C: Into<String>,
              T: Into<String>
    {
        SensorDataSink {
            init_statement: init_statement.into(),
            insert_statement: insert_statement.into(),
            table_name: table_name.into(),
        }
    }
}

impl CrateDBSink for SensorDataSink {
    fn table(&self) -> &str {
        &self.table_name
    }

    fn init(&self) -> &str {
        &self.init_statement
    }

    fn insert(&self) -> &str {
        &self.insert_statement
    }


    fn relay(&self,
             input: Receiver<(Arc<AuthenticatedAgent>, Message)>,
             sink: Cluster,
             batch_size: usize,
             max_timeout: Duration) {

        if let Err(e) = sink.query(self.init(), None::<Box<Nothing>>) {
            warn!("Could not execute CREATE statement: {}", e);
        }
        let insert_stmt = Cow::from(self.insert());
        info!("Using statement '{}' to insert data.", insert_stmt);
        loop {
            let mut v: Vec<(i64, BTreeMap<String, SensorData>, DeviceData)> =
                Vec::with_capacity(batch_size);

            loop {
                if let Ok((agent, msg)) = input.recv_timeout(max_timeout) {
                    let mut sensor_data = BTreeMap::new();

                    for d in msg.data {
                        match d {
                            Measurement::Simple { name, value, unit } => {
                                sensor_data.insert(name,
                                                   SensorData {
                                                       value: json!(value),
                                                       unit: unit,
                                                   });
                            }
                            Measurement::Binary { name, value, unit } => {
                                let mut blob = Cursor::new(value);
                                if let Ok(b) = sink.put(self.table_name.clone(), &mut blob) {
                                    sensor_data.insert(name,
                                                       SensorData {
                                                           value: json!(b.sha1.to_hex()),
                                                           unit: unit,
                                                       });
                                }
                            }
                            Measurement::Tuple { name, value, unit } => {
                                sensor_data.insert(name,
                                                   SensorData {
                                                       value: json!(value),
                                                       unit: unit,
                                                   });
                            }
                            Measurement::Geometry { name, value, unit } => {
                                sensor_data.insert(name,
                                                   SensorData {
                                                       value: json!(value),
                                                       unit: unit,
                                                   });
                            }
                        };
                    }

                    let meta = DeviceData {
                        name: msg.meta.name,
                        agent: agent.name.clone(),
                        role: agent.role.clone(),
                    };
                    v.push((msg.timestamp, sensor_data, meta));
                    if v.len() == batch_size {
                        break;
                    }
                } else {
                    info!("No data received for {:?}. Current queue size: {}",
                          max_timeout,
                          v.len());
                    break;
                }
            }
            let no_items = v.len();
            if no_items > 0 {
                if let Err(e) = sink.bulk_query(insert_stmt.clone(), Box::new(v)) {
                    error!("Error inserting data: {:?}", e);
                } else {
                    debug!("Bulk insert done, {} items inserted", no_items);
                }
            }
        }
    }
}
