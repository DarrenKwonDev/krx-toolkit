use thiserror::Error;

#[derive(Debug, Error)]
pub enum KiwoomError {
    // 전송 계층 에러
    #[error("transport error: {0}")]
    Transport(#[from] reqwest::Error),

    // 400, 500 대의 status 코드 에러
    #[error("http status error: {status}, body: {body}")]
    HttpStatus { status: reqwest::StatusCode, body: String },

    // status는 성공이어도 body 에러 처리하는 뉘앙스의 내용을 보냈을 경우
    #[error("api error: code={code:?}, message={message:?}, raw={raw}")]
    ApiError {
        code: Option<String>,
        message: Option<String>,
        raw: String,
    },

    // 스키마 불일치로 인한 에러
    #[error("decode error: raw={raw}")]
    Decode { raw: String },
}
