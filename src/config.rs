use crate::config::Provider::LocalHost;
use reqwest::Url;
use serde::Deserialize;
use std::{error::Error, fs, path::Path};

#[derive(Deserialize, Debug)]
pub struct Config {
    #[serde(default = "Provider::default")]
    pub provider: Provider,
    pub alert: Alert,
    #[serde(default = "default_interval")]
    pub interval: u16, // unit: second
}

// check every 5 seconds
fn default_interval() -> u16 {
    10
}

#[derive(Deserialize, Debug)]
pub enum Provider {
    AliCloud,
    TencentCloud,
    LocalHost,
}

impl Default for Provider {
    fn default() -> Self {
        LocalHost
    }
}

#[derive(Deserialize, Debug)]
pub struct HeartBeat {
    url: Url,
    secret: String,
}

#[derive(Deserialize, Debug)]
pub struct Alert {
    pub feishu: Feishu,
}

#[derive(Deserialize, Debug)]
pub struct Feishu {
    #[serde(default)]
    pub webhook: String,
    pub secret: String,
}

#[derive(Deserialize, Debug)]
pub struct KeepAlive {
    pub period: u16, // unit: second
    pub client: Client,
}

#[derive(Deserialize, Debug)]
pub struct Client {
    pub url: String,
    pub key: String,
}

#[derive(Deserialize, Debug)]
pub struct Server {
    pub key: String,
    pub max: u16,
}

// load config file in toml format
pub fn load_config(path: &Path) -> Result<String, Box<dyn Error>> {
    let content = fs::read_to_string(path)?;
    println!("{}", content);
    let conf: Config = toml::from_str(&content)?;
    println!("{:?}", conf);
    println!("{}", conf.alert.feishu.webhook);
    Ok(content)
}

#[cfg(test)]
mod test {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_load_config() -> Result<(), Box<dyn Error>> {
        let mut file = NamedTempFile::new()?;
        writeln!(
            file,
            r#"
            [alert.feishu]
            webhook = "https://example.com"
            secret = "ff"
        "#
        )?;
        load_config(&file.path())?;
        Ok(())
    }
}
