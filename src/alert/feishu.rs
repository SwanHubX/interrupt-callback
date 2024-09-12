use std::error::Error;
use super::{Msg, Notice};
use crate::config::Feishu;
use chrono::Utc;
use sha2::Sha256;
use hmac::{Hmac, Mac, digest};
use base64::{engine::general_purpose::STANDARD, Engine as _};
use log::{debug, error};
use serde_json::json;
use reqwest::StatusCode;

impl Notice for Feishu {
    fn send(&self, msg: &Msg) -> Result<(), Box<dyn Error>> {
        let timestamp = Utc::now().timestamp();
        let sign = self.sign(timestamp)?;
        let data = json!({
            "timestamp": timestamp,
            "sign": sign,
            "msg_type": "post",
            "content": {
                "post": {
                    "zh_cn": {
                        "title": msg.code.to_string(),
                        "content": [
                            [{
                                "tag": "text",
                                "text": format!("目标实例：{}", msg.target),
                            }],
                            [{
                                "tag": "text",
                                "text": format!("主机名称：{}", msg.hostname),
                            }],
                            [{
                                "tag": "text",
                                "text": format!("--------\n报警时间：{}", msg.datetime),
                            }]
                        ]
                    }
                }
            }
        });
        debug!("[feishu] request body: {}", data);
        let client = reqwest::blocking::Client::new();
        let res = client.post(self.webhook.to_string()).json(&data).send()?;

        match res.status() {
            StatusCode::OK => Ok(()),
            _ => {
                let text = res.text()?;
                error!("[feishu] sorry, an error happened: {}", text);
                Err(Box::from(text.to_string()))
            }
        }
    }
}

type HmacSha256 = Hmac<Sha256>;

impl Feishu {
    // 1. format string: timestamp(in seconds) + "\n" + secret
    // 2. compute the result by HmacSha256
    // 3. finally, the result is encoded in base64
    // reference: https://open.feishu.cn/document/client-docs/bot-v3/add-custom-bot#3c6592d6
    fn sign(&self, timestamp: i64) -> Result<String, digest::InvalidLength> {
        let str_to_sign = format!("{}\n{}", timestamp.to_string(), self.secret);
        let mac = HmacSha256::new_from_slice(str_to_sign.as_ref())?;
        Ok(STANDARD.encode(mac.finalize().into_bytes()))
    }
}

#[cfg(test)]
mod test {
    use crate::alert::{Code, Target};
    use super::*;

    #[test]
    fn test_send() {
        let fe = Feishu {
            webhook: "https://open.feishu.cn/open-apis/bot/v2/hook/96b876bf-5125-4537-aac0-9ff12dccade7".to_string(),
            secret: "zjVpK38hLf3YrvdnKbq0Qc".to_string(),
        };
        fe.send(&Msg::new(Code::AliCloudInterrupt, Target::Myself("He".to_string()))).expect("TODO: panic message");
        println!("11")
    }

    #[test]
    fn test_sign() {
        let fe = Feishu { webhook: "".to_string(), secret: "Oh, you saw me.".to_string() };
        let sign = fe.sign(1726063290).unwrap();
        assert_eq!(sign, "EG8eZFIOxDlxx0DqlxsEz8YgjXexLF4nmD4seu2WG14=")
    }
}