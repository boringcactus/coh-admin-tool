extern crate actix;
extern crate actix_web;
extern crate env_logger;
extern crate futures;
extern crate odbc;
extern crate serde;
extern crate sha2;

use actix::prelude::*;

mod config;
mod http;
mod db;

fn main() -> Result<(), odbc::DiagnosticRecord> {
    env_logger::init();

    let sys = actix::System::new("coh-admin-tool");

    let conf = config::load();
    let db_conf = conf.clone();

    let addr = SyncArbiter::start(1, move || db::Database::new(&db_conf.clone()).expect("Failed to connect to database"));

    let server = http::Server::new(conf, addr);
    server.start();

    let _ = sys.run();
    Ok(())
}
