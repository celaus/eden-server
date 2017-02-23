
extern crate cratedb;
extern crate serde;
extern crate serde_json;

use self::cratedb::{Nothing, Cluster};
use datasink::CrateDBSink;
use std::sync::mpsc::Receiver;
use handler::Message;
use std::time::Duration;
use std::borrow::Cow;
use auth::AuthenticatedAgent;
use std::sync::Arc;
use std::collections::BTreeMap;

#[derive(Serialize)]
struct SensorData {
    value: f64,
    unit: String,
}


#[derive(Serialize)]
struct DeviceData {
    name: String,
    agent: String,
    role: String,
}


pub struct SensorDataSink {
}


impl SensorDataSink {
    ///
    /// Create a new data sink to relay data towards a sink.
    ///
    pub fn new() -> SensorDataSink {
        SensorDataSink {}
    }
}

impl CrateDBSink for SensorDataSink {
    fn init(&self) -> String {
        "create table if not exists sensors.tp2(ts timestamp, data object, meta object, \
         month as date_trunc('month', ts)) partitioned by (month)"
            .to_string()
    }
    fn insert(&self) -> String {
        "insert into sensors.tp2(ts, data, meta) values(?,?,?)".to_string()
    }


    fn relay(&self,
             data_channel: Receiver<(Arc<AuthenticatedAgent>, Message)>,
             cluster: Cluster,
             batch_size: usize) {
        if let Err(e) = cluster.query(self.init(), None::<Box<Nothing>>) {
            panic!("{:?}", e);
        }
        let max_timeout = Duration::from_secs(90);

        let insert_stmt = Cow::from(self.insert());
        info!("Using statement '{}' to insert data.", insert_stmt);
        loop {
            let mut v: Vec<(i64, BTreeMap<String, SensorData>, DeviceData)> =
                Vec::with_capacity(batch_size);
            loop {
                if let Ok((agent, msg)) = data_channel.recv_timeout(max_timeout) {
                    let mut sensor_data = BTreeMap::new();

                    for d in msg.data {
                        sensor_data.insert(d.sensor,
                                           SensorData {
                                               value: d.value,
                                               unit: d.unit,
                                           });
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
                if let Err(e) = cluster.bulk_query(insert_stmt.clone(), Box::new(v)) {
                    panic!("Error inserting data: {:?}", e);
                } else {
                    debug!("Bulk insert done, {} items inserted", no_items);
                }
            }
        }
    }
}
