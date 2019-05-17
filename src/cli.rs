use crate::config;
use crate::plexpy;
use clap::{crate_authors, crate_description, crate_name, crate_version, App, Arg};
use failure::Error;
use std::env;

enum ConfigType {
    Server,
    Key,
}

fn configure_value(
    env: &str,
    value: ConfigType,
    config: Option<&config::Configuration>,
) -> Result<String, Error> {
    if let Ok(e) = env::var(env) {
        return Ok(e);
    }

    if let Some(config) = config {
        match value {
            ConfigType::Key => return Ok(config.key.clone()),
            ConfigType::Server => return Ok(config.server.clone()),
        }
    }
    Err(failure::err_msg(
        "the api-key and server url must be provided via command line, env variable, or configuartion file",
    ))
}

pub fn execute() -> Result<(), Error> {
    let matches = App::new(crate_name!())
        .author(crate_authors!("\n"))
        .about(crate_description!())
        .version(crate_version!())
        .arg(
            Arg::with_name("server")
                .short("s")
                .long("server")
                .takes_value(true)
                .help("URL of the Tautulli (Plexpy) Server"),
        )
        .arg(
            Arg::with_name("key")
                .short("k")
                .long("key")
                .takes_value(true)
                .help("valid API Key for the server"),
        )
        .get_matches();

    let configuration = config::read_user_config()?;
    let server = match matches.value_of("server") {
        Some(server) => server.to_owned(),
        None => configure_value("PLEXPY_SERVER", ConfigType::Server, configuration.as_ref())?,
    };
    let key = match matches.value_of("key") {
        Some(key) => key.to_owned(),
        None => configure_value("PLEXPY_KEY", ConfigType::Key, configuration.as_ref())?,
    };
    plexpy::get_activity(server, key)?;

    Ok(())
}
