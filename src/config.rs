use serde::Deserialize;
use std::{error::Error, fs, path::Path};
use log::{debug, warn};

#[derive(Deserialize, PartialEq, Debug)]
pub struct Config {
    #[serde(default)]
    pub name: String,
    #[serde(default)]
    pub provider: Provider,
    #[serde(default)]
    pub alert: Alert,
    #[serde(default = "default_interval")]
    pub interval: u16, // unit: second
    #[serde(default)]
    pub keepalive: KeepAlive,
}

// check every 10 seconds
fn default_interval() -> u16 {
    10
}

/*
Provider is used to mark the type of instance.
If AliCloud, request: http://100.100.100.200/latest/meta-data/instance/spot/termination-time,
or TencentCloud, request: metadata.tencentyun.com/latest/meta-data/spot/termination-time,
default is LocalHost, do nothing.
 */
#[derive(Deserialize, Debug, PartialEq)]
pub enum Provider {
    AliCloud,
    TencentCloud,
    LocalHost,
}

impl Default for Provider {
    fn default() -> Self {
        Provider::LocalHost
    }
}

#[derive(Deserialize, Debug, PartialEq)]
pub struct Alert {
    pub feishu: Option<Feishu>,
}

impl Default for Alert {
    fn default() -> Self {
        Alert {
            feishu: None,
        }
    }
}

// feishu open platform
// reference: https://open.feishu.cn/document/client-docs/bot-v3/add-custom-bot#8a6047a
#[derive(Deserialize, Debug, PartialEq)]
pub struct Feishu {
    pub webhook: String,
    pub secret: String,
}

#[derive(Deserialize, Debug, PartialEq)]
pub struct KeepAlive {
    #[serde(default = "default_period")]
    pub period: u16, // unit: second
    pub client: Option<Client>,
    pub server: Option<Server>,
}

impl Default for KeepAlive {
    fn default() -> Self {
        KeepAlive {
            period: 30,
            client: None,
            server: None,
        }
    }
}

#[derive(Deserialize, Debug, PartialEq)]
pub struct Client {
    pub url: String,
    #[serde(default)]
    pub key: String,
}

#[derive(Deserialize, Debug, PartialEq)]
pub struct Server {
    #[serde(default)]
    pub key: String,
    #[serde(default = "default_num")]
    pub num: u16,
}

fn default_period() -> u16 {
    30
}

fn default_num() -> u16 {
    4
}

// load config file in toml format.
// If the file path doesn't exist, it will return default configuration.
pub fn load_config(path: &Path) -> Result<Config, toml::de::Error> {
    let content = fs::read_to_string(path).unwrap_or_else(|err| {
        warn!("{}: use default",err.to_string());
        String::from("")
    });
    let conf: Config = toml::from_str(&content)?;
    debug!("load config: {conf:?}");
    Ok(conf)
}

#[cfg(test)]
mod test {
    use super::*;
    use std::io;
    use std::io::{Write};
    use tempfile::NamedTempFile;

    fn create_temp_file(content: &str) -> Result<NamedTempFile, io::Error> {
        let mut file = NamedTempFile::new()?;
        writeln!(file, "{content}")?;
        Ok(file)
    }

    #[test]
    fn test_load_config() -> Result<(), Box<dyn Error>> {
        let file = create_temp_file(r#"
            provider = "AliCloud"

            [alert.feishu]
            webhook = "https://example.com"
            secret = "111"

            [keepalive]
            period = 5
        "#)?;
        let conf = load_config(Path::new(&file.path()))?;
        assert_eq!(conf.provider, Provider::AliCloud);
        assert_eq!(conf.alert.feishu, Some(Feishu {
            webhook: "https://example.com".to_string(),
            secret: "111".to_string(),
        }));
        assert_eq!(conf.keepalive.period, 5);
        assert_eq!(conf.keepalive.client, None);
        assert_eq!(conf.keepalive.server, None);
        Ok(())
    }

    #[test]
    fn test_load_invalid_file() {
        let conf = load_config(Path::new("")).unwrap();
        let default_conf = Config {
            name: "".to_string(),
            provider: Default::default(),
            alert: Default::default(),
            interval: default_interval(),
            keepalive: Default::default(),
        };
        assert_eq!(conf, default_conf);
    }

    #[test]
    #[should_panic(expected = "wrong configuration")]
    fn error_spec() {
        let file = create_temp_file("name: JSON").unwrap();
        load_config(Path::new(&file.path())).expect("wrong configuration");
    }
}
