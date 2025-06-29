use crate::config::database::Database;

#[derive(serde::Deserialize)]
pub struct Config {
    pub database: Database,
    pub application_port: u16,
}

impl Config {
    pub fn load() -> Config {
        let config = config::Config::builder()
            .add_source(config::File::new(
                "configuration.yaml",
                config::FileFormat::Yaml,
            ))
            .build()
            .expect("failed to read configuration");

        config
            .try_deserialize::<Config>()
            .expect("failed to deserialize config")
    }
}
