use std::time::Duration;

use chrono::TimeZone;
use reqwest::header::{ACCEPT, CONTENT_TYPE, HeaderMap, HeaderValue};
use tokio::sync::{Mutex, RwLock};

use crate::{
    api::kiwoom::{
        error::KiwoomError,
        types::{AccessTokenRequest, AccessTokenResponse},
    },
    constants::{KIWOOM_HTTP_URL_BASE, KIWOOM_TOKEN_REFRESH_BEFORE_MIN},
};
#[derive(Debug, Clone)]
struct TokenState {
    token: Option<String>,
    token_exp: String, // YYYYMMDDHHMMSS (KST)
}
pub struct KiwoomApi {
    base_url: reqwest::Url,
    app_key: String,
    secret_key: String,
    client: reqwest::Client,

    // tokens
    token_state: RwLock<TokenState>,
    refresh_lock: Mutex<()>,
}

impl KiwoomApi {
    pub fn new() -> Result<Self, KiwoomError> {
        let app_key = std::env::var("KIWOOM_APP_KEY").expect("app key missing in .env");
        let secret_key = std::env::var("KIWOOM_SECRET_KEY").expect("secret_key missing in .env");
        let client = Self::get_kiwoom_http_client()?;

        let api = Self {
            base_url: reqwest::Url::parse(KIWOOM_HTTP_URL_BASE)
                .expect("reqwest url parse failed in KIWOOM_HTTP_URL_BASE"),
            app_key: app_key,
            secret_key: secret_key,
            client: client,
            token_state: RwLock::new(TokenState {
                token: None,
                token_exp: String::new(),
            }),
            refresh_lock: Mutex::new(()),
        };

        // CAUTION: create runtime is expensive. should called only once when app booting
        let rt = tokio::runtime::Runtime::new().expect("tokio runtime create failed");
        let token_resp = rt.block_on(api.get_token())?;

        rt.block_on(async {
            let mut state = api.token_state.write().await;
            state.token = Some(token_resp.token);
            state.token_exp = token_resp.expires_dt;
        });

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
            if ok.return_code != 0i32 {
                return Err(KiwoomError::ApiError {
                    code: ok.return_code,
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

    async fn should_refresh(&self) -> bool {
        let state = self.token_state.read().await;
        if state.token.is_none() {
            return true;
        }
        Self::should_refresh_from(&state.token_exp)
    }

    fn should_refresh_from(token_exp: &str) -> bool {
        if token_exp.trim().is_empty() {
            return true;
        }

        let naive = match chrono::NaiveDateTime::parse_from_str(token_exp, "%Y%m%d%H%M%S") {
            Ok(v) => v,
            Err(_) => return true,
        };

        let kst = match chrono::FixedOffset::east_opt(9 * 3600) {
            Some(v) => v,
            None => return true,
        };

        let expires_at = match kst.from_local_datetime(&naive).single() {
            Some(v) => v,
            None => return true,
        };

        let refresh_at = expires_at - chrono::Duration::minutes(KIWOOM_TOKEN_REFRESH_BEFORE_MIN as i64);
        let now_kst = chrono::Utc::now().with_timezone(&kst);
        now_kst >= refresh_at
    }

    async fn ensure_token(&self) -> Result<(), KiwoomError> {
        if !self.should_refresh().await {
            return Ok(());
        }

        let (prev_exp, prev_token_none) = {
            let state = self.token_state.read().await;
            (state.token_exp.clone(), state.token.is_none())
        };

        if !prev_token_none && !Self::should_refresh_from(&prev_exp) {
            return Ok(());
        }

        let _guard = self.refresh_lock.lock().await;

        // 다른 태스크가 먼저 갱신했는지 재확인
        let (cur_exp, cur_token_none) = {
            let state = self.token_state.read().await;
            (state.token_exp.clone(), state.token.is_none())
        };

        if (cur_exp != prev_exp || cur_token_none != prev_token_none)
            && !cur_token_none
            && !Self::should_refresh_from(&cur_exp)
        {
            return Ok(());
        }

        let resp = self.get_token().await?;
        let mut state = self.token_state.write().await;
        state.token = Some(resp.token);
        state.token_exp = resp.expires_dt;

        Ok(())
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
    fn should_refresh_when_exp_empty() {
        assert!(KiwoomApi::should_refresh_from(""));
    }

    #[test]
    fn should_refresh_when_exp_invalid() {
        assert!(KiwoomApi::should_refresh_from("not-a-datetime"));
    }

    #[test]
    fn should_not_refresh_when_exp_is_far_enough() {
        let kst = chrono::FixedOffset::east_opt(9 * 3600).expect("invalid kst offset");
        let exp = (chrono::Utc::now().with_timezone(&kst) + chrono::Duration::minutes(60))
            .format("%Y%m%d%H%M%S")
            .to_string();

        assert!(!KiwoomApi::should_refresh_from(&exp));
    }

    #[test]
    fn should_refresh_when_exp_is_close() {
        let kst = chrono::FixedOffset::east_opt(9 * 3600).expect("invalid kst offset");
        let exp = (chrono::Utc::now().with_timezone(&kst) + chrono::Duration::minutes(1))
            .format("%Y%m%d%H%M%S")
            .to_string();

        assert!(KiwoomApi::should_refresh_from(&exp));
    }

    #[test]
    fn should_refresh_when_already_expired() {
        let kst = chrono::FixedOffset::east_opt(9 * 3600).expect("invalid kst offset");
        let exp = (chrono::Utc::now().with_timezone(&kst) - chrono::Duration::minutes(1))
            .format("%Y%m%d%H%M%S")
            .to_string();

        assert!(KiwoomApi::should_refresh_from(&exp));
    }

    #[tokio::test]
    #[ignore = "requires real KIWOOM credentials + network"]
    async fn create_kiwoomapi_and_fetch_token_integration() {
        let _ = dotenvy::dotenv();

        let api = KiwoomApi::new().expect("failed to create kiwoom api");
        let state = api.token_state.read().await;

        assert!(state.token.is_some());
        assert!(!state.token_exp.is_empty());
    }
}
