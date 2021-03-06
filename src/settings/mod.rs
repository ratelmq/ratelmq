use config::{Config, ConfigError, Environment, File, FileFormat};
use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct MqttSettings {
    pub listeners_tcp: Vec<String>,
}

#[derive(Debug, Deserialize)]
pub struct Settings {
    pub mqtt: MqttSettings,
}

impl Settings {
    pub fn new(config_filename: &str) -> Result<Self, ConfigError> {
        let mut config = Config::new();
        config.merge(File::with_name(config_filename).format(FileFormat::Toml))?;
        config.merge(Environment::with_prefix("ratelmq").separator("__"))?;
        println!("{:?}", &config);
        config.try_into()
    }
}
