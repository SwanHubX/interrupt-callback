use std::error::Error;
use std::io::{Read, Write, BufReader, BufRead, Error as IOError};
use std::net::{Incoming, TcpListener, TcpStream};
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

#[derive(Debug)]
pub struct TcpServer {
    listener: TcpListener,
    key: String, // used to authorize
    name: String,
}

impl TcpServer {
    pub fn new(port: &str, name: &str, key: &str) -> Result<TcpServer, IOError> {
        let listener = TcpListener::bind(format!("0.0.0.0:{port}"))?;
        Ok(TcpServer {
            listener,
            key: key.to_string(),
            name: name.to_string(),
        })
    }

    pub fn incoming(&self) -> Incoming<'_> {
        self.listener.incoming()
    }

    pub fn handle(&self, mut stream: TcpStream) -> Result<Packet, Box<dyn Error>> {
        let buf_reader = BufReader::new(&mut stream);
        let mut buffer = String::new();
        // limit 1024 bytes
        let size = buf_reader.take(1024).read_line(&mut buffer)?;
        debug!("received size: {size}");
        let p: Packet = serde_json::from_str(&buffer)?;
        if p.key != self.key {
            stream.write_all("unauthorized".as_bytes())?;
            return Err(Box::new("403 Unauthorized"));
        }
        // pong
        let packet = Packet {
            key: self.key.clone(),
            name: self.name.clone(),
            msg: "see you next period".to_string(),
        };
        stream.write_all(serde_json::to_string(&packet)?.as_bytes())?;
        Ok(p)
    }
}

#[derive(Debug, PartialEq)]
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
    pub fn new(uri: &str, name: &str) -> Result<TcpClient, String> {
        let u = Url::parse(uri).map_err(|err| format!("invalid uri: {}", err))?;

        if u.scheme() != "ic" || u.username() != "default" {
            return Err("schema or username is illegal".to_string());
        }
        let host = u.host().ok_or("host is required")?;
        // default port is 9080
        let port = u.port().unwrap_or(9080);

        Ok(TcpClient {
            addr: format!("{host}:{port}"),
            key: u.password().unwrap_or("").to_string(),
            name: name.to_string(),
        })
    }


    // keep alive with periodic heartbeat
    pub fn ping(&self, msg: &str) -> Result<Packet, Box<dyn Error>> {
        let mut stream = TcpStream::connect(self.addr.to_string())?;
        let req_packet = Packet {
            key: self.key.clone(),
            name: self.name.clone(),
            msg: msg.to_string(),
        };
        debug!("request packet: {:?}", req_packet);
        // serialize json -> string
        let j = serde_json::to_string(&req_packet)?;

        // ping
        // '\n' marks the end of message
        stream.write(format!("{j}\n").as_bytes())?;

        // pong
        let mut buf = String::new();
        stream.read_to_string(&mut buf)?;
        // deserialize string -> json
        let res_packet: Packet = serde_json::from_str(&buf).map_err(|err| {
            format!("{}: {}", err, buf)
        })?;

        Ok(res_packet)
    }
}

#[cfg(test)]
mod test {
    use std::thread;
    use std::io::{BufRead, BufReader};
    use serde_json::json;
    use super::*;

    #[test]
    #[should_panic(expected = "invalid uri")]
    fn test_tcp_client_invalid_uri() {
        TcpClient::new("127.0.0.1", "").unwrap();
    }

    #[test]
    fn test_tcp_client_ok() {
        let err = TcpClient::new("http://127.0.0.1", "").expect_err("unexpected");
        assert_eq!(err, "schema or username is illegal");
        // is ok
        let client = TcpClient::new("ic://default:apollo@localhost", "").unwrap();
        assert_eq!(client, TcpClient {
            addr: "localhost:9080".to_string(),
            key: "apollo".to_string(),
            name: "".to_string(),
        });
        // empty key
        let client = TcpClient::new("ic://default@127.0.0.1:9999", "io").unwrap();
        assert_eq!(client, TcpClient {
            addr: "127.0.0.1:9999".to_string(),
            key: "".to_string(),
            name: "io".to_string(),
        })
    }

    #[test]
    fn connection_refused() {
        let client = TcpClient::new("ic://default@127.0.0.1:9011", "Q").unwrap();
        assert!(client.ping("are you ok?").is_err())
    }

    fn mock_server(msg: String) -> String {
        // firstly, mock a tcp server
        // :0 means that a random port will be allocated
        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        let addr = listener.local_addr().unwrap().to_string();
        thread::spawn(move || {
            let stream = listener.incoming().next().unwrap();
            let mut s = stream.unwrap();
            handle_packet(&s);
            s.write_all(msg.as_bytes()).unwrap();
        });
        addr
    }

    fn handle_packet(stream: &TcpStream) {
        let buf_reader = BufReader::new(stream);
        let mut buffer = String::new();
        buf_reader.take(1024).read_line(&mut buffer).unwrap();
        let p: Packet = serde_json::from_str(&buffer).unwrap();
        assert_eq!(p.name, "Q");
        assert_eq!(p.key, "coin");
    }

    #[test]
    fn ping_is_ok() {
        let addr = mock_server(json!({
            "name": "server001",
            "msg": "see you next period"
        }).to_string());
        // tcp client
        let client = TcpClient::new(&format!("ic://default:coin@{}", addr), "Q").unwrap();
        let res = client.ping("I'm ok").unwrap();
        assert_eq!(res.name, "server001")
    }

    #[test]
    fn error_response() {
        let addr = mock_server("you are fake".to_string());
        let client = TcpClient::new(&format!("ic://default:coin@{}", addr), "Q").unwrap();
        assert!(client.ping("I'm ok").is_err())
    }
}