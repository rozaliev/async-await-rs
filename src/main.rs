#![feature(conservative_impl_trait)]
#![feature(generator_trait)]
#![feature(generators)]

extern crate mio;
#[macro_use]
extern crate clap;

#[macro_use]
extern crate log;
extern crate env_logger;

#[macro_use]
mod base;
mod net;
mod core;
mod apps;

use clap::App;
use core::Core;

fn main() {
    env_logger::init().unwrap();
    let mut core = Core::new();

    let yaml = load_yaml!("cli.yaml");
    let matches = App::from_yaml(yaml).get_matches();
    let app = match matches.subcommand_name() {
        Some("echo") => apps::echo::serve(),
        Some(n) => unreachable!(),
        None => {
            error!("no subcommand provided");
            return;
        }
    };

    core.run(app);
}