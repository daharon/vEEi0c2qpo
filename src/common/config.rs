use std::net::IpAddr;

use clap::{crate_name, crate_version, value_t_or_exit, Arg};

pub struct Config {
    pub symbol: String,
    pub host: IpAddr,
    pub port: u16,
}

impl Config {
    pub fn new() -> Self {
        let matches = clap::App::new(crate_name!())
            .about("Order Book Merger")
            .version(crate_version!())
            .author("Dan Aharon <dan@aharon.dev>")
            .arg(
                Arg::with_name("host")
                    .short("h")
                    .long("host")
                    .help("Host name or IP address to listen on")
                    .takes_value(true)
                    .value_name("HOSTNAME")
                    .default_value("127.0.0.1"),
            )
            .arg(
                Arg::with_name("port")
                    .short("p")
                    .long("port")
                    .help("Port number to listen on")
                    .takes_value(true)
                    .value_name("PORT")
                    .default_value("8080"),
            )
            .arg(
                Arg::with_name("SYMBOL")
                    .help("The trading symbol, eg. 'ethbtc'")
                    .required(true),
            )
            .get_matches();

        let symbol = value_t_or_exit!(matches.value_of("SYMBOL"), String);
        let host = value_t_or_exit!(matches.value_of("host"), IpAddr);
        let port = value_t_or_exit!(matches.value_of("port"), u16);

        Self { symbol, host, port }
    }
}
