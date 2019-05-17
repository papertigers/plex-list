use failure::Error;
use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct Configuration {
    pub server: String,
    pub key: String,
}

pub fn read_user_config() -> Result<Option<Configuration>, Error> {
    if let Some(mut dir) = dirs::config_dir() {
        // look for for $config/pls.toml
        dir.push("pls.toml");

        if dir.is_file() {
            let contents = std::fs::read(dir)?;
            return Ok(Some(toml::from_slice(&contents)?));
        }
    }
    Ok(None)
}
