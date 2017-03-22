
extern crate cratedb;

use self::cratedb::Cluster;
use std::sync::mpsc::{Receiver, Sender};
use handler::Message;
use auth::AuthenticatedAgent;
use std::sync::Arc;
use std::time::Duration;
use error::ProcessingError;


pub trait CrateDBSink {
    ///
    /// Generates an insert statement.
    ///
    fn insert(&self) -> &str;

    ///
    /// Generate the initialize statement.
    ///
    fn init(&self) -> &str;

    ///
    /// Send messages from the input channel to the cluster.
    ///
    fn relay(&self,
             input: Receiver<(Arc<AuthenticatedAgent>, Message)>,
             sink: Cluster,
             bulk_size: usize,
             max_timeout: Duration);
}

pub trait StreamProcessor<I, O> {
    fn process(&self, input: I) -> Result<O, ProcessingError>;

    ///
    /// Send messages from the input channel to the output channel.
    ///
    fn relay(&self, input: Receiver<I>, output: Sender<O>, max_timeout: Duration) {
        loop {
            if let Ok(i) = input.recv_timeout(max_timeout) {
                match self.process(i) {
                    Ok(o) => output.send(o).unwrap(),
                    Err(e) => error!("Could not process {:?}", e),
                }
            } else {
                error!{"No data received within {:?}.", max_timeout};
            }
        }
    }
}
