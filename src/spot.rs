use std::time::Duration;
use log::error;
use reqwest::{Error, StatusCode, blocking::Client};

struct Spot {
    client: Client,
}

impl Spot {
    pub fn new() -> Spot {
        Spot {
            client: Client::new()
        }
    }

    // query the interruption events of a preemptive instance of aliyun (alias ecs)
    // fixed url: GET http://100.100.100.200/latest/meta-data/instance/spot/termination-time
    // there are the following results
    // 0: it means that the instance will be released in a few minutes.
    // 1: normal
    // 2. unknown error
    // reference: https://help.aliyun.com/zh/ecs/use-cases/query-the-interruption-events-of-preemptible-instances
    pub fn query_ecs(&self, uri: Option<String>) -> Result<i8, Error> {
        let url = uri.unwrap_or_else(|| {
            "http://100.100.100.200/latest/meta-data/instance/spot/termination-time".to_string()
        });
        // limit 1s
        let res = self.client.get(url)
            .timeout(Duration::from_secs(1))
            .send()?;

        match res.status() {
            StatusCode::OK => Ok(0),
            StatusCode::NOT_FOUND => Ok(1),
            _ => {
                error!("[ecs] unknown error: {}", res.text()?);
                Ok(2)
            }
        }
    }

    pub fn query_cvm() {}
}

#[cfg(test)]
mod test {
    use super::*;

    // network error
    #[test]
    fn test_query_ecs_err() {
        let spot = Spot::new();
        let err = spot.query_ecs(None).expect_err("timeout");
        assert!(err.is_timeout());
    }

    fn test_query_ecs(status: usize, code: i8) {
        let mut server = mockito::Server::new();

        let mock = server.mock("GET", "/spot")
            .with_status(status)
            .create();
        let spot = Spot::new();
        let c = spot.query_ecs(Some(format!("{}/spot", server.url()))).unwrap();
        assert_eq!(c, code);
        mock.assert();
    }

    // (true) will be released
    #[test]
    fn test_query_ecs_0() {
        test_query_ecs(200, 0);
    }

    #[test]
    fn test_query_ecs_1() {
        test_query_ecs(404, 1);
    }

    #[test]
    fn test_query_ecs_2() {
        test_query_ecs(500, 2);
    }
}