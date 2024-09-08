use std::{fs, io};
use std::path::Path;
use serde::Deserialize;

#[derive(Deserialize, Debug)]
pub struct Config {
    provider: Provider,
    alert: Alert,
}

#[derive(Deserialize, Debug)]
enum Provider {
    AliCloud,
    TencentCloud,
    LocalHost,
}

#[derive(Deserialize, Debug)]
struct Alert {
    feishu: Feishu,
}

#[derive(Deserialize, Debug)]
struct Feishu {
    webhook: String,
    secret: String,
}

// load config file in toml format
pub fn load_config(path: &Path) -> Result<String, io::Error> {
    let con = fs::read_to_string(path)?;
    println!("{}", con);
    Ok(con)
}

#[cfg(test)]
mod test {
    use super::*;
    use std::fs::File;
    use std::io::Write;
    use tempfile::tempdir;


    #[test]
    fn test_load_config() {
        let dir = tempdir().unwrap();
        let path = dir.path().join("config.toml");
        let mut f = File::create(&path).unwrap();
        writeln!(f, "f001").unwrap();
        println!("{:?}", dir.path());
        load_config(&path).unwrap();
    }
}