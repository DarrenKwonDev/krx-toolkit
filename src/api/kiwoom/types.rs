use serde::{Deserialize, Serialize};

// -------------------------------
// 에러 코드는 응답시
// 	"return_code":0,
//	"return_msg":"정상적으로 처리되었습니다"
// -------------------------------

// -------------------------------
// 접근토큰발급
// https://openapi.kiwoom.com/guide/apiguide?dummyVal=0
// -------------------------------
#[derive(Debug, Clone, Serialize)]
pub struct AccessTokenRequest {
    pub grant_type: String,
    pub appkey: String,
    pub secretkey: String,
}
impl AccessTokenRequest {
    pub fn new(appkey: String, secretkey: String) -> Self {
        Self {
            grant_type: "client_credentials".to_owned(),
            appkey,
            secretkey,
        }
    }
}
#[derive(Debug, Clone, Deserialize)]
pub struct AccessTokenResponse {
    pub expires_dt: String,
    pub token_type: String,
    pub token: String,
    pub return_code: String,
    pub return_msg: String,
}
