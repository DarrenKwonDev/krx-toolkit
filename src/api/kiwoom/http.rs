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
pub struct MasterStockPage {
    pub body: serde_json::Value,
    pub cont_yn: Option<String>,
    pub next_key: Option<String>,
}
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

#[derive(Debug, Clone)]
pub struct BuyStockRequest {
    pub dmst_stex_tp: String,
    pub stk_cd: String,
    pub ord_qty: u64,
    pub ord_uv: Option<u64>,
    pub trde_tp: String,
    pub cond_uv: Option<u64>,
}

#[derive(Debug, Clone)]
pub struct BuyStockResponse {
    pub ord_no: Option<String>,
    pub dmst_stex_tp: Option<String>,
    pub raw: serde_json::Value,
}

impl KiwoomApi {
    pub async fn new() -> Result<Self, KiwoomError> {
        let app_key = std::env::var("KIWOOM_APP_KEY").expect("app key missing in .env");
        let secret_key = std::env::var("KIWOOM_SECRET_KEY").expect("secret_key missing in .env");
        let client = Self::get_kiwoom_http_client()?;

        let api = Self {
            base_url: reqwest::Url::parse(KIWOOM_HTTP_URL_BASE).expect("reqwest url parse failed in KIWOOM_HTTP_URL_BASE"),
            app_key: app_key,
            secret_key: secret_key,
            client: client,
            token_state: RwLock::new(TokenState {
                token: None,
                token_exp: String::new(),
            }),
            refresh_lock: Mutex::new(()),
        };

        let token_resp = api.get_token().await?;
        {
            let mut state = api.token_state.write().await;
            state.token = Some(token_resp.token);
            state.token_exp = token_resp.expires_dt;
        }

        Ok(api)
    }

    pub async fn access_token(&self) -> Result<String, KiwoomError> {
        self.ensure_token().await?;

        let state = self.token_state.read().await;
        state.token.clone().ok_or_else(|| KiwoomError::Decode {
            raw: "access token missing after ensure_token".to_owned(),
        })
    }

    pub async fn fetch_master_stock(
        &self,
        mrkt_tp: &str,
        cont_yn: Option<&str>,
        next_key: Option<&str>,
    ) -> Result<MasterStockPage, KiwoomError> {
        let token = self.access_token().await?;
        let mut req = self
            .client
            .post(self.url(routes::종목정보))
            .header("authorization", format!("Bearer {token}"))
            .header("api-id", tr::종목정보리스트)
            .json(&serde_json::json!({ "mrkt_tp": mrkt_tp }));
        if let Some(v) = cont_yn {
            req = req.header("cont-yn", v);
        }
        if let Some(v) = next_key {
            req = req.header("next-key", v);
        }
        let resp = req.send().await.map_err(KiwoomError::Transport)?;
        let status = resp.status();
        let resp_cont_yn = resp
            .headers()
            .get("cont-yn")
            .and_then(|v| v.to_str().ok())
            .map(ToOwned::to_owned);
        let resp_next_key = resp
            .headers()
            .get("next-key")
            .and_then(|v| v.to_str().ok())
            .map(ToOwned::to_owned);
        let text = resp.text().await.map_err(KiwoomError::Transport)?;
        if !status.is_success() {
            return Err(KiwoomError::HttpStatus { status, body: text });
        }
        let body =
            serde_json::from_str::<serde_json::Value>(&text).map_err(|_| KiwoomError::Decode { raw: text.clone() })?;
        if let Some(code) = body.get("return_code").and_then(|v| v.as_i64()) {
            if code != 0 {
                let msg = body.get("return_msg").and_then(|v| v.as_str()).map(ToOwned::to_owned);
                return Err(KiwoomError::ApiError {
                    code: code as i32,
                    message: msg,
                    raw: text,
                });
            }
        }
        Ok(MasterStockPage {
            body,
            cont_yn: resp_cont_yn,
            next_key: resp_next_key,
        })
    }

    pub async fn buy_stock(&self, req_data: BuyStockRequest) -> Result<BuyStockResponse, KiwoomError> {
        let token = self.access_token().await?;
        let payload = serde_json::json!({
            "dmst_stex_tp": req_data.dmst_stex_tp,
            "stk_cd": req_data.stk_cd,
            "ord_qty": req_data.ord_qty.to_string(),
            "ord_uv": req_data.ord_uv.map(|v| v.to_string()),
            "trde_tp": req_data.trde_tp,
            "cond_uv": req_data.cond_uv.map(|v| v.to_string()),
        });

        let resp = self
            .client
            .post(self.url(routes::주문))
            .header("authorization", format!("Bearer {token}"))
            .header("api-id", tr::주식매수주문)
            .json(&payload)
            .send()
            .await
            .map_err(KiwoomError::Transport)?;

        let status = resp.status();
        let text = resp.text().await.map_err(KiwoomError::Transport)?;
        if !status.is_success() {
            return Err(KiwoomError::HttpStatus { status, body: text });
        }

        let body =
            serde_json::from_str::<serde_json::Value>(&text).map_err(|_| KiwoomError::Decode { raw: text.clone() })?;
        if let Some(code) = body.get("return_code").and_then(|v| v.as_i64()) {
            if code != 0 {
                let msg = body.get("return_msg").and_then(|v| v.as_str()).map(ToOwned::to_owned);
                return Err(KiwoomError::ApiError {
                    code: code as i32,
                    message: msg,
                    raw: text,
                });
            }
        }

        Ok(BuyStockResponse {
            ord_no: body.get("ord_no").and_then(|v| v.as_str()).map(str::to_owned),
            dmst_stex_tp: body.get("dmst_stex_tp").and_then(|v| v.as_str()).map(str::to_owned),
            raw: body,
        })
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

#[allow(dead_code)]
mod routes {
    pub const 접근토큰발급: &str = "/oauth2/token";
    pub const 접근토큰폐기: &str = "/oauth2/revoke";
    pub const 계좌: &str = "/api/dostk/acnt";
    pub const 종목정보: &str = "/api/dostk/stkinfo";
    pub const 주문: &str = "/api/dostk/ordr";
}

#[allow(dead_code)]
mod tr {
    pub const 계좌평가현황요청: &str = "kt00004"; // 누적손익률
    pub const 당일실현손익상세요청: &str = "ka10077"; // trade 별 실현 손익/수익률
    pub const 종목정보리스트: &str = "ka10099"; // 마스터
    pub const 주식매수주문: &str = "kt10000";
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

        let api = KiwoomApi::new().await.expect("failed to create kiwoom api");
        let state = api.token_state.read().await;

        assert!(state.token.is_some());
        assert!(!state.token_exp.is_empty());
    }
}
