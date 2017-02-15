
extern crate cratedb;

use self::cratedb::Cluster;
use std::sync::mpsc::Receiver;
use handler::Message;
use auth::AuthenticatedAgent;
use std::sync::Arc;

pub trait CrateDBSink {

    ///
    /// Generates an insert statement.
    ///
    fn insert(&self) -> String;

    ///
    /// Generate the initialize statement.
    ///
    fn init(&self) -> String;

    ///
    /// Send messages from the input channel to the cluster.
    ///
    fn relay(&self, data_channel: Receiver<(Arc<AuthenticatedAgent>, Message)>, cluster: Cluster, bulk_size: usize);
}
