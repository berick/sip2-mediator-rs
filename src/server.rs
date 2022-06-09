use super::conf::Config;
use super::session::Session;
use log::{error, warn, info, debug};
use sip2;
use std::io::prelude::*;
use std::net;
use std::net::TcpListener;
use std::net::TcpStream;
use std::thread;
use std::time::Duration;
use threadpool::ThreadPool;

pub struct Server {
    config: Config,
}

impl Server {

    pub fn new(config: Config) -> Server {
        Server {
            config,
        }
    }

    pub fn serve(&mut self) {

        info!("SIP2Meditor server staring up");

        let pool = ThreadPool::new(self.config.max_clients);

        let bind = format!("{}:{}", self.config.sip_address, self.config.sip_port);

        let listener = TcpListener::bind(bind).expect("Error starting SIP server");

        for stream in listener.incoming() {
            match stream {
                Ok(s) => self.dispatch(&pool, s),
                Err(e) => error!("Error accepting TCP connection {}", e),
            }
        }

        info!("SIP2Mediator shutting down; waiting for threads to complete");

        pool.join();
    }

    /// Pass the new SIP TCP stream off to a thread for processing.
    fn dispatch(&self, pool: &ThreadPool, stream: TcpStream) {

        debug!(
            "Accepting new SIP connection; active={} pending={}",
            pool.active_count(),
            pool.queued_count()
        );

        let threads = pool.active_count() + pool.queued_count();

        if threads >= self.config.max_clients {

            warn!(
                "Max clients={} reached.  Rejecting new connections",
                self.config.max_clients
            );

            if let Err(e) = stream.shutdown(net::Shutdown::Both) {
                error!("Error shutting down SIP TCP connection: {}", e);
            }

            return;
        }

        // Hand the stream off for processing.
        let conf = self.config.clone();
        pool.execute(|| Session::run(conf, stream));
    }
}

