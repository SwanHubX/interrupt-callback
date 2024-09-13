use std::error::Error;
use std::io::{Read, Write};
use std::net::{TcpListener, TcpStream};
use std::sync::Mutex;
use log::{debug, error};
use reqwest::Url;
use serde::{Deserialize, Serialize};


#[derive(Serialize, Deserialize, Debug)]
struct Packet {
    #[serde(default)]
    key: String, // password for authorization
    name: String, // server identifier from sender
    #[serde(default)]
    msg: String,
}

pub struct TcpServer {
    listener: TcpListener,
    key: String,
}

impl TcpServer {
    pub fn handle(&self) {
        todo!()
    }
}

struct TcpClient {
    addr: String, // example: 127.0.0.1:9080
    key: String, // optional, default is empty
    name: String, // not modifiable
}

impl TcpClient {
    // the uri format is ic://default:{key}@{host}:{port}
    // - the schema must be 'ic://'
    // - currently, username must be 'default'
    // - key and port are allowed to be empty
    //
    // example: ic://default@127.0.0.1:9080, ic://default:password@49.15.34.11:9080
    pub fn new(uri: String, name: String) -> Result<TcpClient, String> {
        let u = Url::parse(&uri).map_err(|err| format!("invalid uri: {}", err))?;

        if u.scheme() != "ic" || u.username() != "default" {
            return Err("schema or username is illegal".to_string());
        }
        let host = u.host().ok_or("host is required")?;
        // default port is 9080
        let port = u.port().unwrap_or(9080);

        Ok(TcpClient {
            addr: format!("{host}:{port}"),
            key: u.password().unwrap_or("").to_string(),
            name,
        })
    }


    pub fn ping(&self, msg: String) -> Result<Packet, Box<dyn Error>> {
        let mut stream = TcpStream::connect(self.addr.to_string())?;
        let req_packet = Packet {
            key: self.key.to_string(),
            name: self.name.to_string(),
            msg,
        };
        // serialize json -> string
        let j = serde_json::to_string(&req_packet)?;

        // ping
        // '\n' marks the end of message
        stream.write(format!("{j}\n").as_bytes())?;

        // pong
        let mut buf = String::new();
        stream.read_to_string(&mut buf)?;
        // deserialize string -> json
        let res_packet: Packet = serde_json::from_str(&buf)?;

        Ok(res_packet)
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    #[should_panic(expected = "invalid uri")]
    fn test_tcp_client_invalid_uri() {
        TcpClient::new("127.0.0.1".to_string(), "hi".to_string()).unwrap();
    }
}