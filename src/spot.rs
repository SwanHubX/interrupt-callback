use log::error;
use reqwest::{blocking::Client, Error, StatusCode};
use std::time::Duration;

pub struct Spot {
    client: Client,
}

impl Spot {
    pub fn new() -> Spot {
        Spot {
            client: Client::new(),
        }
    }

    // there are the following results
    // 0: it means that the instance will be released in a few minutes.
    // 1: normal
    // 2. unknown error
    fn query(&self, url: String) -> Result<i8, Error> {
        // limit 1s
        let res = self
            .client
            .get(url)
            .timeout(Duration::from_secs(1))
            .send()?;

        match res.status() {
            StatusCode::OK => Ok(0),
            StatusCode::NOT_FOUND => Ok(1),
            _ => {
                error!("[spot instance] unknown error: {}", res.text()?);
                Ok(2)
            }
        }
    }

    // query the interruption events of a preemptive instance of aliyun (alias ecs)
    // fixed url: GET http://100.100.100.200/latest/meta-data/instance/spot/termination-time
    // there are the following results
    // 0: it means that the instance will be released in a few minutes.
    // 1: normal
    // 2. unknown error
    // reference: https://help.aliyun.com/zh/ecs/use-cases/query-the-interruption-events-of-preemptible-instances
    pub fn query_ecs(&self) -> Result<i8, Error> {
        self.query(
            "http://100.100.100.200/latest/meta-data/instance/spot/termination-time".to_string(),
        )
    }

    // the spot instance of tencentcloud (alias cvm)
    // fixed url: GET http://metadata.tencentyun.com/latest/meta-data/spot/termination-time
    // reference: https://cloud.tencent.com/document/product/213/37970
    pub fn query_cvm(&self) -> Result<i8, Error> {
        self.query(
            "http://metadata.tencentyun.com/latest/meta-data/spot/termination-time".to_string(),
        )
    }
}

#[cfg(test)]
mod test {
    use super::*;

    // network error
    #[test]
    fn test_query_err() {
        let spot = Spot::new();
        let err = spot
            .query("http://100.100.100.200".to_string())
            .expect_err("timeout");
        assert!(err.is_timeout());
    }

    fn test_query(status: usize, code: i8) {
        let mut server = mockito::Server::new();

        let mock = server.mock("GET", "/spot").with_status(status).create();
        let spot = Spot::new();
        let c = spot.query(format!("{}/spot", server.url())).unwrap();
        assert_eq!(c, code);
        mock.assert();
    }

    // (true) will be released
    #[test]
    fn test_query_0() {
        test_query(200, 0);
    }

    #[test]
    fn test_query_1() {
        test_query(404, 1);
    }

    #[test]
    fn test_query_2() {
        test_query(500, 2);
    }
}
