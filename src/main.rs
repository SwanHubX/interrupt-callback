mod config;
mod alert;
mod spot;
mod keepalive;

use crate::alert::Target::Myself;
use crate::alert::{Alert, AlertMap, Code, Msg};
use crate::keepalive::{TcpClient, TcpServer};
use config::Provider;
use env_logger::Builder;
use log::{debug, error, info, warn, LevelFilter};
use spot::Spot;
use std::path::Path;
use std::sync::Arc;
use std::{env, thread, time::Duration};


/*
 The main function will launch the following service base on config
 - check the status of the spot instance regularly
 - a tcp client with timed heartbeat
 - launch a tcp server that monitor the status of multiple clients (default :9080)

 all the environment variables
 - CONFIG_PATH: path to the configuration file. optional, default is empty
 - SERVER_PORT: the port of tcp server. default is 9080
 */
fn main() {
    Builder::new().filter_level(LevelFilter::Info).init();
    // 1. load config. note: default path is /etc/ic/config.toml
    let conf_path = env::var("CONFIG_PATH").unwrap_or("/etc/ic/config.toml".to_string());
    let conf = config::load_config(Path::new(&conf_path)).unwrap();
    debug!("the config: {:?}", conf);
    // 2. create an alert for notification
    let mut map = AlertMap::new();
    if let Some(fe) = conf.alert.feishu {
        map.insert("feishu".to_string(), Box::new(fe));
    }
    let alert = Arc::new(Alert::new(map));
    // the unique id - {name}@{hostname}
    let name = format!("{}@{}", conf.name, alert::hostname());
    info!("the name is {name}");

    // all services run in a child thread, so we need to wait before the main ends
    let mut handles = vec![];

    // 3. monitor the status of the server
    let spot = Spot::new();
    let sp = SpotPatrol::new(conf.interval as u64, name.clone(), Arc::clone(&alert));
    match conf.provider {
        Provider::AliCloud => {
            info!("create a thread used to monitor aliyun ecs");
            let h = thread::spawn(move || {
                sp.patrol(|| spot.query_ecs(), Code::AliCloudInterrupt);
            });
            handles.push(h);
        }
        Provider::TencentCloud => {
            info!("create a thread used to monitor tencentcloud cvm");
            let h = thread::spawn(move || {
                sp.patrol(|| spot.query_cvm(), Code::TencentCloudInterrupt)
            });
            handles.push(h);
        }
        _ => (),
    };

    // 4. if configured, turn on a client with timed heartbeat
    let period = conf.keepalive.period;
    if let Some(c) = conf.keepalive.client {
        match TcpClient::new(&c.uri, &name.clone()) {
            Ok(client) => {
                let h = thread::spawn(move || {
                    // super loop
                    loop {
                        match client.ping("I am active") {
                            Ok(p) => info!("client - {}: {}", p.name, p.msg),
                            Err(err) => error!("client - ping error: {err}")
                        };
                        thread::sleep(Duration::from_secs(period as u64));
                    }
                });
                handles.push(h);
                info!("start a tcp client");
            }
            Err(err) => error!("failed to create a client: {err}")
        };
    }
    // 5. if configured, launch a server
    if let Some(s) = conf.keepalive.server {
        let port: u16 = match env::var("SERVER_PORT") {
            Ok(s) => s.parse::<u16>().unwrap_or(9080),
            Err(_) => 9080
        };
        match TcpServer::new(port, &name.clone(), period, s) {
            Ok(server) => {
                let alert = Arc::clone(&alert);
                let h = thread::spawn(move || server.run(alert));
                handles.push(h);
                info!("start a tcp server");
            }
            Err(err) => error!("failed to create a server: {err}")
        };
    };

    for h in handles {
        h.join().unwrap();
    }
    warn!("please check the configuration file, it is all over");
}


struct SpotPatrol {
    interval: u64, // patrol interval
    name: String,
    alert: Arc<Alert>,
}

impl SpotPatrol {
    fn new(interval: u64, name: String, alert: Arc<Alert>) -> SpotPatrol {
        SpotPatrol {
            interval,
            name,
            alert,
        }
    }

    fn patrol<F>(&self, query: F, code: Code)
    where
        F: Fn() -> Result<i8, reqwest::Error>,
    {
        // super loop
        loop {
            match query() {
                Ok(i) => {
                    match i {
                        // will be released in a few minutes
                        0 => {
                            // clone a copy
                            let alert = Arc::clone(&self.alert);
                            let name = self.name.clone();
                            let code = code.clone();
                            thread::spawn(move || alert.send(&Msg::new(code, Myself(name))));
                            // end the thread
                            break;
                        }
                        1 => info!("everything is ok with this server"),
                        u => error!("unknown error: {u}"),
                    };
                }
                Err(err) => {
                    error!("spot - query error: {}", err);
                }
            };
            // delay
            thread::sleep(Duration::from_secs(self.interval));
        }
    }
}


