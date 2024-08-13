use reqwest::{Client, Response};
use serde::Deserialize;
use std::collections::HashMap;
use std::time::Duration;
use url::Url;
use utils::async_trait::async_trait;
use utils::{anyhow, chrono, BResult as Result};
use utils::tracing::{debug, trace};

#[async_trait]
pub trait BaseApi {
    async fn get_json(&self, base_urls: &str, path: &str, params: Option<&HashMap<&str, &str>>) -> Result<serde_json::Value>;
    fn get_headers(&self) -> HashMap<String, String>;
}

pub struct AppClient {
    client: Client,
    headers: HashMap<String, String>,
    timeout: Duration,
    base_api_url: String,
    base_live_api_url: String,
    base_play_info_api_url: String,
    app_key: String,
    app_secret: String,
}

impl AppClient {
    pub fn new(headers: Option<HashMap<String, String>>) -> Self {
        let default_headers = vec![
            ("Accept-Encoding", "gzip, deflate, br"),
            ("Accept-Language", "zh-CN,zh;q=0.8,zh-TW;q=0.7,zh-HK;q=0.5,en;q=0.3,en-US;q=0.2"),
            ("Accept", "application/json, text/plain, */*"),
            ("Cache-Control", "no-cache"),
            ("Connection", "keep-alive"),
            ("Origin", "https://live.bilibili.com"),
            ("Pragma", "no-cache"),
            ("User-Agent", "Mozilla/5.0 BiliDroid/6.64.0 (bbcallen@gmail.com) os/android model/Unknown mobi_app/android build/6640400 channel/bili innerVer/6640400 osVer/6.0.1 network/2"),
        ].into_iter().map(|(k, v)| (k.to_string(), v.to_string())).collect();

        let headers = headers.unwrap_or(default_headers);

        Self {
            client: Client::new(),
            headers,
            timeout: Duration::from_secs(10),
            base_api_url: "https://api.bilibili.com".to_string(),
            base_live_api_url: "https://api.live.bilibili.com".to_string(),
            base_play_info_api_url: "https://api.live.bilibili.com".to_string(),
            app_key: "1d8b6e7d45233436".to_string(),
            app_secret: "560c52ccd288fed045859ed18bffd973".to_string(),
        }
    }

    async fn get_json_res(&self, url: &str, params: Option<&HashMap<String, String>>) -> Result<serde_json::Value> {
        let req = self.client.get(url).headers(self.headers.clone().into());
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

    fn signed(&self, mut params: HashMap<&str, &str>) -> HashMap<String, String> {
        params.insert("appkey", &self.app_key);
        let mut params_vec: Vec<_> = params.into_iter().collect();
        params_vec.sort_by(|a, b| a.0.cmp(b.0));
        let query = serde_urlencoded::to_string(&params_vec).unwrap();
        let sign = format!("{:x}", md5::compute(format!("{}{}", query, self.app_secret)));
        params_vec.push(("sign", sign.as_str()));
        params_vec.into_iter().map(|(k, v)| (k.to_string(), v.to_string())).collect()
    }

    pub fn update_heads(&mut self, headers: HashMap<String, String>) {
        self.headers.extend(headers)
    }

}

#[async_trait]
impl BaseApi for AppClient {
    async fn get_json(&self, base_urls: &str, path: &str, params: Option<&HashMap<String, String>>) -> Result<serde_json::Value> {
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


impl AppClient {

    pub async fn get_room_play_infos(&self, room_id: i32, qn: i32) -> Result<serde_json::Value> {
        let path = "/xlive/app-room/v2/index/getRoomPlayInfo";
        let params = self.signed(HashMap::from([
            ("actionKey", "appkey"),
            ("build", "6640400"),
            ("channel", "bili"),
            ("codec", "0,1"),
            ("device", "android"),
            ("device_name", "Unknown"),
            ("disable_rcmd", "0"),
            ("dolby", "1"),
            ("format", "0,1,2"),
            ("free_type", "0"),
            ("http", "1"),
            ("mask", "0"),
            ("mobi_app", "android"),
            ("need_hdr", "0"),
            ("no_playurl", "0"),
            ("only_audio", "0"),
            ("only_video", "0"),
            ("platform", "android"),
            ("play_type", "0"),
            ("protocol", "0,1"),
            ("qn", &qn.to_string()),
            ("room_id", &room_id.to_string()),
            ("ts", &format!("{}", chrono::Utc::now().timestamp())),
        ]));
        self.get_json(&"https://api.live.bilibili.com", path, Some(&params)).await
    }

    pub async fn get_info_by_room(&self, room_id: usize) -> Result<serde_json::Value> {
        let path = "/xlive/app-room/v1/index/getInfoByRoom";
        let params = self.signed(HashMap::from([
            ("actionKey", "appkey"),
            ("build", "6640400"),
            ("channel", "bili"),
            ("device", "android"),
            ("mobi_app", "android"),
            ("platform", "android"),
            ("room_id", &room_id.to_string()),
            ("ts", &format!("{}", chrono::Utc::now().timestamp())),
        ]));
        self.get_json("https://api.live.bilibili.com", path, Some(&params)).await
    }

    pub async fn get_user_info(&self, uid: i32) -> Result<serde_json::Value> {
        let path = "/x/v2/space";
        let params = self.signed(HashMap::from([
            ("build", "6640400"),
            ("channel", "bili"),
            ("mobi_app", "android"),
            ("platform", "android"),
            ("ts", &format!("{}", chrono::Utc::now().timestamp())),
            ("vmid", &uid.to_string()),
        ]));
        self.get_json(&"https://app.bilibili.com", path, Some(&params)).await
    }
    pub async fn get_danmu_info(&self, room_id: i32) -> Result<serde_json::Value> {
        let path = "/xlive/app-room/v1/index/getDanmuInfo";
        let params = self.signed(HashMap::from([
            ("actionKey", "appkey"),
            ("build", "6640400"),
            ("channel", "bili"),
            ("device", "android"),
            ("mobi_app", "android"),
            ("platform", "android"),
            ("ts", &format!("{}", chrono::Utc::now().timestamp())),
            ("room_id", &room_id.to_string())
        ]));
        self.get_json(&"https://app.bilibili.com", path, Some(&params)).await
    }
}
