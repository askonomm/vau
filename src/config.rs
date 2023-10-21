use crate::{Error, ROOT_DIR};
use serde::Deserialize;
use std::fs;

#[derive(Deserialize, Debug)]
pub struct ConfigDataDSLWhenIs {
    pub key: String,
    pub equals: String,
}

#[derive(Deserialize, Debug)]
pub struct ConfigDataDSLWhenIsNot {
    pub key: String,
    pub equals: String,
}

#[derive(Deserialize, Debug)]
pub struct ConfigDataDSLWhenHas {
    pub key: String,
}

#[derive(Deserialize, Debug)]
pub struct ConfigDataDSLWhenHasNot {
    pub key: String,
}

#[derive(Deserialize, Debug)]
pub struct ConfigDataDSLWhenMatches {
    pub key: String,
    pub regex: String,
}

#[derive(Deserialize, Debug)]
pub struct ConfigDataDSLSort {
    pub key: String,
    pub order: String,
}

#[derive(Deserialize, Debug)]
pub struct ConfigDataDSL {
    pub name: String,
    pub collection: String,
    pub when_is: Option<ConfigDataDSLWhenIs>,
    pub when_is_not: Option<ConfigDataDSLWhenIsNot>,
    pub when_has: Option<ConfigDataDSLWhenHas>,
    pub when_has_not: Option<ConfigDataDSLWhenHasNot>,
    pub when_matches: Option<ConfigDataDSLWhenMatches>,
    pub sort: Option<ConfigDataDSLSort>,
    pub limit: Option<usize>,
    pub first: Option<bool>,
    pub last: Option<bool>,
}

#[derive(Deserialize, Debug)]
pub struct ConfigPagesDSLPage {
    pub path: String,
}

#[derive(Deserialize, Debug)]
pub struct ConfigPagesDSL {
    pub collection: Option<String>,
    pub page: ConfigPagesDSLPage,
    pub template: String,
}

#[derive(Deserialize, Debug)]
pub struct Config {
    pub data: Option<Vec<ConfigDataDSL>>,
    pub pages: Option<Vec<ConfigPagesDSL>>,
}

pub fn read_config() -> Result<Config, Error> {
    let config_path = format!("{}config.toml", ROOT_DIR);
    let config = fs::read_to_string(config_path)?;
    let parsed_config: Config = toml::from_str(&config)?;

    Ok(parsed_config)
}
