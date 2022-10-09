use std::env;
use std::net::Ipv4Addr;
use simple_logger::SimpleLogger;
use anyhow::Result;
use log::{debug, error, LevelFilter};

#[derive(Debug)]
pub struct Config {
    pub keys: Vec<String>,
    pub ip_addr: Ipv4Addr,
    pub port: u16,
}

pub fn setup() -> Result<Config> {
    SimpleLogger::new()
        .with_level(LevelFilter::Warn)
        .with_module_level("generic_cache_server", LevelFilter::Trace)
        .with_utc_timestamps()
        .init()?;

    let ip_addr: Ipv4Addr = match env::var("ADDRESS") {
        Ok(addr) => match addr.parse::<Ipv4Addr>() {
            Ok(addr) => addr,
            Err(err) => {
                error!("Invalid ADDRESS: {:?}", err);
                debug!("Using 0.0.0.0 as ADDRESS");
                Ipv4Addr::new(0, 0, 0, 0)
            }
        },
        Err(err) => {
            error!("Invalid/missing ADDRESS: {:?}", err);
            debug!("Using 0.0.0.0 as ADDRESS");
            Ipv4Addr::new(0, 0, 0, 0)
        }
    };

    let port = match env::var("PORT") {
        Ok(port_str) => match port_str.parse::<u16>() {
            Ok(port) => port,
            Err(err) => {
                error!("Invalid PORT ({port_str}): {:?}", err);
                debug!("Using 8080 as PORT");
                8080
            }
        },
        Err(err) => {
            error!("Invalid/missing PORT: {:?}", err);
            debug!("Using 8080 as PORT");
            8080
        }
    };

    let mut keys = vec![];
    for (key, value) in env::vars().into_iter() {
        if key.starts_with("GCS_KEY_") {
            keys.push(value);
        }
    }

    if keys.is_empty() {
        error!("No access keys provided, using test keys");
        keys.push(String::from("test"));
    }

    let config = Config {
        ip_addr,
        port,
        keys
    };

    Ok(config)
}