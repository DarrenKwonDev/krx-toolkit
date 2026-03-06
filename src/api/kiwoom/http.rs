use std::time::Duration;

use reqwest::header::{ACCEPT, CONTENT_TYPE, HeaderMap, HeaderValue};

use crate::{
    api::kiwoom::{
        error::KiwoomError,
        types::{AccessTokenRequest, AccessTokenResponse},
    },
    constants::KIWOOM_HTTP_URL_BASE,
};

pub struct KiwoomApi {
    base_url: reqwest::Url,
    app_key: String,
    secret_key: String,
    token: Option<String>,
    token_exp: String,
    client: reqwest::Client,
}

impl KiwoomApi {
    pub fn new() -> Result<Self, KiwoomError> {
        let app_key = std::env::var("KIWOOM_APP_KEY").expect("app key missing in .env");
        let secret_key = std::env::var("KIWOOM_SECRET_KEY").expect("secret_key missing in .env");
        let client = Self::get_kiwoom_http_client().expect("fail to create kiwoom http client");

        let mut api = Self {
            base_url: reqwest::Url::parse(KIWOOM_HTTP_URL_BASE)
                .expect("reqwest url parse failed in KIWOOM_HTTP_URL_BASE"),
            app_key: app_key,
            secret_key: secret_key,
            token: None,
            token_exp: String::new(),
            client: client,
        };

        let rt = tokio::runtime::Runtime::new().expect("tokio runtime create failed");
        let token_resp = rt.block_on(api.get_token())?;
        api.token = Some(token_resp.token);
        api.token_exp = token_resp.expires_dt;
        Ok(api)
    }

    // --------------------------------
    // private
    // --------------------------------
    async fn get_token(&self) -> Result<AccessTokenResponse, KiwoomError> {
        let body = AccessTokenRequest::new(self.app_key.clone(), self.secret_key.clone());

        let resp = self
            .client
            .post(self.url(routes::접근토큰발급))
            .json(&body)
            .send()
            .await
            .map_err(KiwoomError::Transport)?;

        let status = resp.status();
        let text = resp.text().await.map_err(KiwoomError::Transport)?;

        if !status.is_success() {
            return Err(KiwoomError::HttpStatus { status, body: text });
        }

        if let Ok(ok) = serde_json::from_str::<AccessTokenResponse>(&text) {
            if ok.return_code != "0" {
                return Err(KiwoomError::ApiError {
                    code: Some(ok.return_code),
                    message: Some(ok.return_msg),
                    raw: text.clone(),
                });
            }
            return Ok(ok);
        }

        Err(KiwoomError::Decode { raw: text })
    }

    fn url(&self, path: &str) -> reqwest::Url {
        self.base_url
            .join(path.trim_start_matches('/'))
            .expect("invalid kiwoom path")
    }

    fn get_kiwoom_http_client() -> Result<reqwest::Client, reqwest::Error> {
        let mut headers = HeaderMap::new();
        headers.insert(ACCEPT, HeaderValue::from_static("application/json;charset=UTF-8"));
        headers.insert(CONTENT_TYPE, HeaderValue::from_static("application/json;charset=UTF-8"));
        reqwest::Client::builder()
            .default_headers(headers)
            .connect_timeout(Duration::from_secs(2))
            .timeout(Duration::from_secs(8))
            .pool_idle_timeout(Duration::from_secs(30))
            .pool_max_idle_per_host(8)
            .tcp_nodelay(true)
            .user_agent("krx-toolkit/0.1.0")
            .build()
    }
}

mod routes {
    pub const 접근토큰발급: &str = "/oauth2/token";
    pub const 접근토큰폐기: &str = "/oauth2/revoke";
    pub const 계좌: &str = "/api/dostk/acnt";
    pub const 종목정보: &str = "/api/dostk/stkinfo";
    pub const 주문: &str = "/api/dostk/ordr";
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_create_kiwoomapi() {
        let _ = dotenvy::dotenv();
        let has_key = std::env::var("KIWOOM_APP_KEY").is_ok();
        let has_secret = std::env::var("KIWOOM_SECRET_KEY").is_ok();
        if !(has_key && has_secret) {
            return;
        }

        let api = KiwoomApi::new().expect("fail create kiwoom api");
        assert_eq!(api.base_url.as_str(), KIWOOM_HTTP_URL_BASE);
        assert!(api.token.is_none());
        assert!(api.token_exp.is_empty());
    }
}
