
extern crate cratedb;

use self::cratedb::Cluster;
use dto::TemperaturePressureReading;
use std::sync::mpsc::Receiver;

pub trait CrateDBSink {
    fn insert(&self) -> String;
    fn init(&self) -> String;
    fn relay(&self, data_channel: Receiver<TemperaturePressureReading>, cluster: Cluster);
}

pub struct TemperaturePressureDataSink {
    batch_size: usize,
}
impl TemperaturePressureDataSink {
    pub fn new(batch_size: usize) -> TemperaturePressureDataSink {
        TemperaturePressureDataSink { batch_size: batch_size }
    }
}

impl CrateDBSink for TemperaturePressureDataSink {
    fn init(&self) -> String {
        "create table if not exists sensors.tp(ts timestamp, temperature double, pressure double,  \
         month as date_trunc('month', ts)) partitioned by (month)"
            .to_string()
    }
    fn insert(&self) -> String {
        "insert into sensors.tp(ts, temperature, pressure) values(?,?,?)".to_string()
    }


    fn relay(&self, data_channel: Receiver<TemperaturePressureReading>, mut cluster: Cluster) {
        let mut current_batch = 0;
        if let Err(e) = cluster.query(&self.init(), None) {
            panic!("{:?}", e);
        }

        let insert_stmt: String = self.insert();
        loop {
            let mut v: Vec<(u64, f64, f64)> = Vec::with_capacity(self.batch_size);

            loop {
                if let Ok(msg) = data_channel.recv() {
                    v.push((msg.ts, msg.t, msg.p));
                    current_batch += 1;
                    if current_batch == self.batch_size {
                        break;
                    }
                }
            }
            if let Err(e) = cluster.bulk_query(&self.insert(), Box::new(v)) {
                panic!("Error inserting data: {:?}", e);
            } else {
                current_batch = 0;
                info!("Bulk insert done")
            }

        }
    }
}
