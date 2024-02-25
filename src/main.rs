use std::env;
use std::thread;
use std::time::Duration;

const BASE_URL: &str = "http://metadata.tencentyun.com/latest/meta-data/spot/termination-time";
const TOKEN: &str = "ogUSAsQicRW1pOVfuq-rO";

fn main() {
    let args: Vec<String> = env::args().collect();
    let webhook_url = &args[1]; // webhook url

    loop {
        // request metadata
        let resp = reqwest::blocking::get(BASE_URL).unwrap();
        let status = resp.status().as_u16();
        if status == 200 {
            // webhook notification
            let client = reqwest::blocking::Client::new();
            let resp = client.post(webhook_url)
                .body("")
                .header("Authorization", TOKEN)
                .send().unwrap();
            println!("status: {}", resp.status().as_u16());
            break;
        }
        // delay 3s
        thread::sleep(Duration::from_secs(3));
    }
}