use reqwest::{Client, Response};
use serde::Deserialize;
use std::collections::HashMap;
use std::hash::Hash;
use std::str::FromStr;
use std::time::Duration;
use async_trait::async_trait;
use anyhow::{anyhow, Result};
use reqwest::header::{HeaderMap, HeaderName, HeaderValue};
use tracing::debug;

#[async_trait]
pub trait BaseApi {
    async fn get_json(&self, base_urls: &str, path: &str, params: Option<&HashMap<&str, &str>>) -> Result<serde_json::Value>;
    fn get_headers(&self) -> HashMap<String, String>;
}

pub struct WebClient {
    client: Client,
    headers: HashMap<String, String>,
    timeout: Duration,
    base_api_url: String,
    base_live_api_url: String,
    base_play_info_api_url: String,
}
fn convert_headers(headers: &HashMap<String, String>) -> HeaderMap {
    let mut header_map = HeaderMap::new();
    for (key, value) in headers {
        let header_name = HeaderName::from_str(key).unwrap();
        let header_value = HeaderValue::from_str(value).unwrap();
        header_map.insert(header_name, header_value);
    }
    header_map
}

impl WebClient {
    pub fn new(headers: Option<HashMap<String, String>>) -> Self {
        let default_headers = vec![
            ("Accept-Encoding", "gzip, deflate, br"),
            ("Accept-Language", "zh-CN,zh;q=0.8,zh-TW;q=0.7,zh-HK;q=0.5,en;q=0.3,en-US;q=0.2"),
            ("Accept", "application/json, text/plain, */*"),
            ("Cache-Control", "no-cache"),
            ("Connection", "keep-alive"),
            // ("Origin", "https://live.bilibili.com"),
            ("Pragma", "no-cache"),
            ("User-Agent", "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/119.0.0.0 Safari/537.36"),
        ].into_iter().map(|(k, v)| (k.to_string(), v.to_string())).collect();

        let headers = headers.unwrap_or(default_headers);

        Self {
            client: Client::builder()
                .gzip(true)
                .build().unwrap(),
            headers,
            timeout: Duration::from_secs(10),
            base_api_url: "https://api.bilibili.com".to_string(),
            base_live_api_url: "http://api.live.bilibili.com".to_string(),
            base_play_info_api_url: "https://api.live.bilibili.com".to_string(),
        }
    }

    async fn get_json_res(&self, url: &str, params: Option<&HashMap<&str, &str>>) -> Result<serde_json::Value> {
        let req = self.client.get(url).headers(convert_headers(&self.headers));
        let req = if let Some(params) = params {
            req.query(params)
        } else {
            req
        };
        let res = req.send().await?.json::<serde_json::Value>().await?;
        debug!("Request: {:?}", url);
        debug!("Response: {:?}", res);
        Ok(res)
    }

    pub fn update_heads(&mut self, headers: HashMap<String, String>) {
        self.headers.extend(headers)
    }
}

#[async_trait]
impl BaseApi for WebClient {
    async fn get_json(&self, base_urls: &str, path: &str, params: Option<&HashMap<&str, &str>>) -> Result<serde_json::Value> {
        let mut exception = None;
        let url = format!("{}{}", base_urls, path);
        match self.get_json_res(&url, params).await {
            Ok(json_res) => return Ok(json_res),
            Err(e) => {
                debug!("Failed to get json from {}: {:?}", url, e);
                exception = Some(e);
            }
        }
        Err(exception.unwrap_or_else(|| anyhow::anyhow!("No base urls provided")))
    }

    fn get_headers(&self) -> HashMap<String, String> {
        self.headers.clone()
    }
}


impl WebClient {
    pub async fn room_init(&self, room_id: i32) -> Result<serde_json::Value> {
        let path = "/room/v1/Room/room_init";
        let id = room_id.to_string();
        let params = HashMap::from([
            ("id", id.as_str())
        ]);
        self.get_json(&self.base_live_api_url, path, Some(&params)).await
    }

    pub async fn get_room_play_infos(&self, room_id: usize, qn: i32) -> Result<serde_json::Value> {
        let path = "/xlive/web-room/v2/index/getRoomPlayInfo";
        let qn = qn.to_string();
        let room_id = room_id.to_string();
        let params = HashMap::from([
            ("room_id", room_id.as_str()),
            ("ptype", "8"),
            ("platform", "web"),
            ("codec", "0,1"),
            ("format", "0,1,2"),
            ("protocol", "0,1"),
            ("qn", qn.as_str())
        ]);
        self.get_json(&self.base_live_api_url, path, Some(&params)).await
    }

    pub async fn get_info_by_room(&self, room_id: usize) -> Result<serde_json::Value> {
        let path = "/xlive/web-room/v1/index/getInfoByRoom";
        let room_id = room_id.to_string();
        let params = HashMap::from([
            ("room_id", room_id.as_str())
        ]);
        self.get_json(&self.base_live_api_url, path, Some(&params)).await
    }

    pub async fn get_info(&self, room_id: usize) -> Result<serde_json::Value> {
        let path = "/room/v1/Room/get_info";
        let room_id = room_id.to_string();
        let params = HashMap::from([
            ("room_id", room_id.as_str())
        ]);
        self.get_json(&self.base_live_api_url, path, Some(&params)).await
    }

    pub async fn get_timestamp(&self, room_id: usize) -> Result<serde_json::Value> {
        let path = "/av/v1/Time/getTimestamp";
        let params = HashMap::from([
            ("platform", "pc")
        ]);
        self.get_json(&self.base_live_api_url, path, Some(&params)).await
    }

    pub async fn get_user_info(&self, uid: i32) -> Result<serde_json::Value> {
        let path = "/x/space/wbi/acc/info";
        let uid = uid.to_string();
        let params = HashMap::from([
            ("mid", uid.as_str()),
        ]);
        self.get_json(&"https://app.bilibili.com", path, Some(&params)).await
    }
    pub async fn get_danmu_info(&self, room_id: i32) -> Result<serde_json::Value> {
        let path = "/xlive/web-room/v1/index/getDanmuInfo";
        let room_id = room_id.to_string();
        let params = HashMap::from([
            ("room_id", room_id.as_str())
        ]);
        self.get_json(&"https://app.bilibili.com", path, Some(&params)).await
    }

    pub async fn get_nav(&self, room_id: i32) -> Result<serde_json::Value> {
        let path = "/x/web-interface/nav";
        self.get_json(&"https://app.bilibili.com", path, None).await
    }
}


#[cfg(test)]
mod test {
    use anyhow::Result;
    use crate::api::WebClient;
    use tokio::test;

    #[tokio::test]
    async fn test_get_room_play_infos() -> Result<()> {
        let client = WebClient::new(None);
        // let room_id = 9922197;
        let room_id = 2297410; // 替换为有效的房间 ID
        let qn = 10000; // 替换为有效的质量编号

        let result = client.get_room_play_infos(room_id, qn).await;

        match result {
            Ok(json) => {
                println!("Response: {:?}", json);
                assert!(json.is_object(), "Expected JSON object");
            }
            Err(e) => {
                eprintln!("Error: {:?}", e);
                assert!(false, "Request failed");
            }
        }
        Ok(())
    }

    #[tokio::test]
    async fn test_get_info_by_room() -> Result<()> {
        let client = WebClient::new(None);
        let room_id = 2297410; // 替换为有效的房间 ID

        let result = client.get_info_by_room(room_id).await;

        match result {
            Ok(json) => {
                println!("Response: {:?}", json);
                assert!(json.is_object(), "Expected JSON object");
            }
            Err(e) => {
                eprintln!("Error: {:?}", e);
                assert!(false, "Request failed");
            }
        }
        Ok(())
    }
}