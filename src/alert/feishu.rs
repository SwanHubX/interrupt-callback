// reference: https://open.feishu.cn/document/client-docs/bot-v3/add-custom-bot#8a6047a

use std::io::Error;
use super::{Msg, Notice};
use crate::config::Feishu;
use chrono::Utc;
use sha2::Sha256;
use hmac::{Hmac, Mac, digest};
use base64::{engine::general_purpose::STANDARD, Engine as _};

impl Notice for Feishu {
    fn send(&self, msg: &Msg) -> Result<(), Error> {
        todo!()
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
        let mut mac = HmacSha256::new_from_slice(str_to_sign.as_ref())?;
        Ok(STANDARD.encode(mac.finalize().into_bytes()))
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_send() {
        println!("11")
    }

    #[test]
    fn test_sign() {
        let fe = Feishu { webhook: "".to_string(), secret: "Oh, you saw me.".to_string() };
        let sign = fe.sign(1726063290).unwrap();
        assert_eq!(sign, "EG8eZFIOxDlxx0DqlxsEz8YgjXexLF4nmD4seu2WG14=")
    }
}