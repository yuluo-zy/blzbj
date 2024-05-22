use std::collections::HashMap;
use serde::{Deserialize, Serialize};
use utils::async_trait::async_trait;
use utils::error::ApiRequestError;
use utils::reqwest::Client;
use utils::{error};
use utils::reqwest::header::{HeaderMap, HeaderName, HeaderValue};

pub static BASE_HEADERS: &[(&str, &str)] = &[
    ("Accept-Encoding", "gzip, deflate, br"),
    ("Accept-Language", "zh-CN,zh;q=0.8,zh-TW;q=0.7,zh-HK;q=0.5,en;q=0.3,en-US;q=0.2"),
    ("Accept", "application/json, text/plain, */*"),
    ("Cache-Control", "no-cache"),
    ("Connection", "keep-alive"),
    ("Origin", "https://live.bilibili.com"),
    ("Pragma", "no-cache"),
    ("User-Agent", "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/119.0.0.0 Safari/537.36"),
];

#[derive(Debug, Deserialize)]
pub struct JsonResponse<T> {
    code: i32,
    message: Option<String>,
    msg: Option<String>,
    data: Option<T>,
}

pub type QualityNumber = i32;

#[derive(Debug, Serialize, Deserialize)]
pub struct ResponseData {
    pub live_status: Option<i32>,
    pub uid: Option<i32>,
    pub timestamp: Option<i64>,
}

#[async_trait]
pub trait BaseApi: Sync + Send {
    fn new(client: Client, headers: HeaderMap, room_id: Option<i32>) -> Self;
    async fn get_json_res<T: for<'de> Deserialize<'de>>(&self, url: &str, params: &HashMap<String, String>) -> Result<JsonResponse<T>, ApiRequestError>;
    async fn get_json<T: for<'de> Deserialize<'de>>(
        &self,
        base_urls: &[String],
        path: &str,
        params: &HashMap<String, String>,
    ) -> Result<JsonResponse<T>, ApiRequestError> {
        if base_urls.is_empty() {
            return Err(ApiRequestError::NoBaseUrls);
        }

        let mut exception = None;
        for base_url in base_urls {
            let url = format!("{}{}", base_url, path);
            match self.get_json_res(&url, params).await {
                Ok(res) => return Ok(res),
                Err(e) => {
                    exception = Some(e);
                    error!("request json error: {}", e.to_string())
                }
            }
        }

        Err(exception.unwrap())
    }

    fn check_response<T>(&self, json_res: &JsonResponse<T>) -> Result<(), ApiRequestError> {
        if json_res.code != 0 {
            let message = json_res.message.clone().or_else(|| json_res.msg.clone()).unwrap_or_default();
            return Err(ApiRequestError::ApiError(json_res.code, message));
        }
        Ok(())
    }
}

pub struct WebApi {
    client: Client,
    headers: HeaderMap,
    room_id: Option<i32>,
    base_api_urls: Vec<String>,
    base_live_api_urls: Vec<String>,
    base_play_info_api_urls: Vec<String>,
}

#[async_trait]
impl BaseApi for WebApi {
    fn new(client: Client, mut header: HeaderMap, room_id: Option<i32>) -> Self {
        for &item in BASE_HEADERS {
            let header_name = HeaderName::from_bytes(item.0.as_bytes())?;
            let header_value = HeaderValue::from_str(item.1)?;
            header.insert(header_name, header_value);
        }
        Self {
            client,
            headers,
            room_id,
            base_api_urls: vec!["https://api.bilibili.com".to_string()],
            base_live_api_urls: vec!["https://api.live.bilibili.com".to_string()],
            base_play_info_api_urls: vec!["https://api.live.bilibili.com".to_string()],
        }
    }

    async fn  get_json_res<T: for<'de> Deserialize<'de>>(&self, url: &str, params: &HashMap<String, String>) -> Result<JsonResponse<T>, ApiRequestError> {
        let res = self.client.get(url).headers(self.headers.clone())
            .query(params).send().await?;
        let json_res = res.json().await?;
        self.check_response(&json_res)?;
        Ok(json_res)
    }
}

impl WebApi {
    // pub async fn room_init(&self, room_id: i32) -> Result<ResponseData, ApiRequestError> {
    //     let path = "/room/v1/Room/room_init";
    //     let mut params = HashMap::new();
    //     params.insert("id".to_string(), room_id.to_string());
    //
    //     let json_res = self.get_json(&self.base_live_api_urls, path, &params).await?;
    //     Ok(serde_json::from_value(json_res.data.unwrap())?)
    // }
    //
    // pub async fn get_room_play_infos(&self, room_id: i32, qn: QualityNumber) -> Result<Vec<ResponseData>, ApiRequestError> {
    //     let path = "/xlive/web-room/v2/index/getRoomPlayInfo";
    //     let mut params = HashMap::new();
    //     params.insert("room_id".to_string(), room_id.to_string());
    //     params.insert("protocol".to_string(), "0,1".to_string());
    //     params.insert("format".to_string(), "0,1,2".to_string());
    //     params.insert("codec".to_string(), "0,1".to_string());
    //     params.insert("qn".to_string(), qn.to_string());
    //     params.insert("platform".to_string(), "web".to_string());
    //     params.insert("ptype".to_string(), "8".to_string());
    //
    //     let json_res = self.get_json(&self.base_play_info_api_urls, path, &params).await?;
    //     Ok(serde_json::from_value(json_res.data.unwrap())?)
    // }
    //
    pub async fn get_info_by_room(&self, room_id: i32) -> Result<ResponseData, ApiRequestError> {
        let path = "/xlive/web-room/v1/index/getInfoByRoom";
        let mut params = HashMap::new();
        params.insert("room_id".to_string(), room_id.to_string());

        let json_res = self.get_json(&self.base_live_api_urls, path, &params).await?;
        Ok(serde_json::from_value(json_res.data.unwrap())?)
    }
    //
    // pub async fn get_info(&self, room_id: i32) -> Result<ResponseData, ApiRequestError> {
    //     let path = "/room/v1/Room/get_info";
    //     let mut params = HashMap::new();
    //     params.insert("room_id".to_string(), room_id.to_string());
    //
    //     let json_res = self.get_json(&self.base_live_api_urls, path, &params).await?;
    //     Ok(serde_json::from_value(json_res.data.unwrap())?)
    // }
    //
    // pub async fn get_timestamp(&self) -> Result<i64, ApiRequestError> {
    //     let path = "/av/v1/Time/getTimestamp";
    //     let mut params = HashMap::new();
    //     params.insert("platform".to_string(), "pc".to_string());
    //
    //     let json_res = self.get_json(&self.base_live_api_urls, path, &params).await?;
    //     Ok(json_res.data.unwrap()["timestamp"].as_i64().unwrap())
    // }
    //
    // pub async fn get_user_info(&self, uid: i32) -> Result<ResponseData, ApiRequestError> {
    //     let path = "/x/space/wbi/acc/info";
    //     let mut params = HashMap::new();
    //     params.insert("mid".to_string(), uid.to_string());
    //
    //     let json_res = self.get_json(&self.base_api_urls, path, &params).await?;
    //     Ok(serde_json::from_value(json_res.data.unwrap())?)
    // }
    //
    // pub async fn get_danmu_info(&self, room_id: i32) -> Result<ResponseData, ApiRequestError> {
    //     let path = "/xlive/web-room/v1/index/getDanmuInfo";
    //     let mut params = HashMap::new();
    //     params.insert("id".to_string(), room_id.to_string());
    //
    //     let json_res = self.get_json(&self.base_live_api_urls, path, &params).await?;
    //     Ok(serde_json::from_value(json_res.data.unwrap())?)
    // }
    //
    // pub async fn get_nav(&self) -> Result<ResponseData, ApiRequestError> {
    //     let path = "/x/web-interface/nav";
    //
    //     let mut params = HashMap::new();
    //     let json_res = self.get_json(&self.base_api_urls, path, &params).await?;
    //     Ok(serde_json::from_value(json_res.data.unwrap())?)
    // }
}

