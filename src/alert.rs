mod feishu;

use chrono::{FixedOffset, Local};
use log::{error, info};
use std::collections::HashMap;
use std::{fmt, io};
use sysinfo::System;

// all the events must transfer Msg instance
// - code is the type of event
// - target is the source of event
#[derive(Debug)]
pub struct Msg {
    code: Code,
    target: Target,
    hostname: String,
    datetime: String,
}

impl Msg {
    pub fn new(code: Code, target: Target) -> Self {
        Msg {
            code,
            target,
            hostname: hostname(),
            datetime: now(),
        }
    }
}

// china standard time（UTC +8）
// example: 2024-09-11 16:43:21
fn now() -> String {
    let offset = FixedOffset::east_opt(8 * 60 * 60).unwrap();
    Local::now().with_timezone(&offset).format("%Y-%m-%d %H:%M:%S").to_string()
}

// system's host name
fn hostname() -> String {
    System::host_name().unwrap_or_else(|| "".to_string())
}

#[derive(Debug)]
pub enum Code {
    // the spot instance of AliCloud will terminate.
    AliCloudInterrupt,
    // the spot instance of TencentCloud will terminate.
    TencentCloudInterrupt,
    // the server is offline because of network, power outage, etc.
    // detect with another server
    Offline,
    Online,
}

impl fmt::Display for Code {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Code::AliCloudInterrupt => write!(f, "阿里云服务器释放通知"),
            Code::TencentCloudInterrupt => write!(f, "腾讯云服务器释放通知"),
            Code::Offline => write!(f, "服务器离线通知"),
            Code::Online => write!(f, "服务器上线通知"),
        }
    }
}

#[derive(Debug)]
pub enum Target {
    Myself(String),
    Another(String),
}


pub trait Notice {
    fn send(&self, msg: &Msg) -> Result<(), io::Error>;
}

type AlertMap = HashMap<String, Box<dyn Notice>>;

pub struct Alert {
    integrations: AlertMap,
}

impl Alert {
    fn new(integrations: AlertMap) -> Alert {
        Alert {
            integrations
        }
    }

    // this method will return the result of sending with HashMap
    fn send(&self, msg: &Msg) -> HashMap<String, bool> {
        let mut result = HashMap::new();
        for (name, notice) in self.integrations.iter() {
            info!("send an alert to {name}");
            // if it failed, do nothing
            let mut is_ok = true;
            notice.send(msg).unwrap_or_else(|err| {
                error!("fail to send to {name}: {}", err.to_string());
                is_ok = false;
            });
            result.insert(name.to_string(), is_ok);
        }
        result
    }
}


#[cfg(test)]
mod test {
    use super::*;
    use chrono::NaiveDateTime;
    use std::collections::HashMap;
    use std::io::{Error, ErrorKind};

    #[test]
    fn test_now() {
        let datetime = now();
        let fmt = "%Y-%m-%d %H:%M:%S";
        assert!(NaiveDateTime::parse_from_str(&datetime, fmt).is_ok());
    }

    #[test]
    fn test_hostname() {
        assert!(!hostname().is_empty());
    }

    #[test]
    fn test_code() {
        assert_eq!("阿里云服务器释放通知", Code::AliCloudInterrupt.to_string());
        assert_eq!("服务器上线通知", Code::Online.to_string());
    }

    // mock notices object for testing
    struct Success {}
    struct Failure {}

    impl Notice for Success {
        fn send(&self, _msg: &Msg) -> Result<(), Error> {
            Ok(())
        }
    }

    impl Notice for Failure {
        fn send(&self, _msg: &Msg) -> Result<(), Error> {
            Err(Error::new(ErrorKind::TimedOut, "test"))
        }
    }

    #[test]
    fn test_send_alert() {
        let mut integrations: AlertMap = HashMap::new();
        integrations.insert(String::from("s"), Box::new(Success {}));
        integrations.insert(String::from("f"), Box::new(Failure {}));

        let length = integrations.len();
        let alert = Alert::new(integrations);
        let msg = Msg::new(Code::Online, Target::Myself("hi".to_string()));
        let res = alert.send(&msg);

        assert_eq!(length, res.len());
        assert!(res.get("s").unwrap());
        assert!(!res.get("f").unwrap());
    }
}