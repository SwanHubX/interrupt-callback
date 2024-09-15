use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::{env, thread};
use std::error::Error;
use crate::config;
use crate::alert::{Alert, Code, Msg, Target::Another};
use crate::keepalive::{Packet, TcpServer};


type WhiteList = HashMap<String, u8>;

pub struct Watchdog {
    name: String,
    conf: config::Server,
    alert: Arc<Alert>,
    mu: Arc<Mutex<WhiteList>>,
}

impl Watchdog {
    pub fn new(name: &str, conf: config::Server, alert: Arc<Alert>) -> Watchdog {
        let list: WhiteList = HashMap::new();
        let mu = Arc::new(Mutex::new(list));
        Watchdog {
            name: name.to_string(),
            conf,
            alert,
            mu,
        }
    }

    pub fn run(&self) -> Result<(), Box<dyn Error>> {
        let port: u16 = match env::var("SERVER_PORT") {
            Ok(s) => s.parse::<u16>().unwrap_or(9080),
            Err(_) => 9080
        };
        let server = TcpServer::new(port, &self.name, &self.conf.key)?;
        let app = Arc::new(server);
        for stream in app.incoming() {
            let s = stream?;
            let alert = Arc::clone(&self.alert);
            let app = Arc::clone(&app);
            thread::spawn(move || {
                let p = app.handle(s).unwrap();
                // if let Some(code) = self.register(&p).unwrap() {
                alert.send(&Msg::new(Code::AliCloudInterrupt, Another(p.name)));
                // };
            });
        }
        Ok(())
    }

    // fn register(&mut self, p: &Packet) -> Result<Option<Code>, Box<dyn Error>> {
    //     let mut list = self.mu.lock()?;
    //     let mut res: Option<Code> = None;
    //     if let Some(v) = list.get(&p.name) {
    //         if *v == 0 {
    //             res = Some(Code::Online);
    //         }
    //     };
    //     // start over
    //     list.insert(p.name.to_string(), self.conf.num);
    //     Ok(res)
    // }

    // pub fn patrol(&mut self, alert: &Alert) -> Result<(), Box<dyn Error>> {
    //     let mut list = self.mu.lock()?;
    //     for (k, v) in list.iter_mut() {
    //         if *v == 0 {
    //             continue;
    //         }
    //         *v = v.saturating_sub(1);
    //         if *v == 0 {
    //             thread::spawn(move || {
    //                 alert.send(&Msg::new(Code::Offline, Another(k.to_string())));
    //             });
    //         }
    //     };
    //
    //     Ok(())
    // }
}


#[cfg(test)]
mod test {
    #[test]
    fn test() {
        println!("11")
    }
}