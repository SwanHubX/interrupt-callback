#![allow(unused)]
use std::env;
use std::thread;
use std::time::Duration;
use std::process::{Command, Output};
use std::path::{Path, PathBuf};
use serde_json::json;

// const BASE_URL: &str = "https://course.rs/basic/result-error/panic.html";
const BASE_URL: &str = "http://metadata.tencentyun.com/latest/meta-data/spot/termination-time";
// const BASE_URL: &str = "http://metadata.tencentyun.com/latest/meta-data/payment/create-time";
const INSTANCE_ID_URL: &str = "http://metadata.tencentyun.com/latest/meta-data/instance-id";
const INSTANCE_NAME_URL: &str = "http://metadata.tencentyun.com/latest/meta-data/instance-name";
const DEFAULT_TOKEN: &str = "ogUSAsQicRW1pOVfuq-rO";
const SCRIPT_NAME: &str = "interrupt_callback.sh";

fn main() {
    let args: Vec<String> = env::args().collect();
    let webhook_url = &args[1]; // webhook url
    println!("webhook url: {}", webhook_url);

    let token = match args.len() {
        3 => args[2].clone(),
        _ => DEFAULT_TOKEN.to_owned(),
    };

    loop {
        // request metadata
        let resp = reqwest::blocking::get(BASE_URL).unwrap();
        let status = resp.status().as_u16();
        println!("termination-time status: {:?}", status);

        if status == 200 {
            println!("\n WARNING ! The instance will go to an end. \n");
            // instance_id, instance_name, release_time
            let instance_id = reqwest::blocking::get(INSTANCE_ID_URL).unwrap();
            let instance_name = reqwest::blocking::get(INSTANCE_NAME_URL).unwrap();
            let release_time = reqwest::blocking::get(BASE_URL).unwrap();
            let body = json!({
                "instance_id": instance_id.text().unwrap(),
                "instance_name": instance_name.text().unwrap(),
                "release_time": release_time.text().unwrap(),
            });

            // webhook notification
            let client = reqwest::blocking::Client::new();
            let resp = client.post(webhook_url)
                .body(body.to_string())
                .header("Authorization", DEFAULT_TOKEN)
                .send().unwrap();
            println!("request body: {:?}", body.to_string());
            println!("response body: {:?}", resp.text().unwrap());

            // 运行 shell 脚本
            // let current_dir = env::current_dir().expect("Failed to get current directory").join(SCRIPT_NAME);
            // let output = execute_shell_script_from_file(current_dir.to_str().unwrap());
            // let stdout = String::from_utf8_lossy(&output.stdout);
            // println!("\n ---------- stdout ---------- \n {:?}", stdout);

            break;
        }

        // delay 3s
        thread::sleep(Duration::from_secs(3));
    }
}


fn execute_shell_script_from_file(script_file: &str) -> Output {
    // 使用 Command::new 创建一个 Command 对象，指定要执行的命令
    let output = Command::new("sh")
        .arg(script_file) // 在 sh 中执行的命令，指定脚本文件路径
        .output() // 执行命令并获取输出
        .expect("Failed to execute command");

    // 返回命令执行结果
    output
}