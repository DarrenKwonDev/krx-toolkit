use futures_util::{SinkExt, StreamExt};
use serde_json::Value;
use tokio::net::TcpStream;
use tokio_tungstenite::{MaybeTlsStream, WebSocketStream, connect_async, tungstenite::Message};

use crate::{api::kiwoom::error::KiwoomError, constants::KIWOOM_WS_URL_BASE, ts_dbg};

type WsStream = WebSocketStream<MaybeTlsStream<TcpStream>>;

#[derive(Debug, Clone, serde::Serialize)]
pub struct WsRegData {
    pub item: Vec<String>,
    pub r#type: Vec<String>,
}

pub struct KiwoomWs {
    url: String,
    socket: Option<WsStream>,
    connected: bool,
}

impl KiwoomWs {
    pub fn new() -> Self {
        Self {
            url: format!("{}:10000/api/dostk/websocket", KIWOOM_WS_URL_BASE),
            socket: None,
            connected: false,
        }
    }

    pub fn is_connected(&self) -> bool {
        self.connected
    }

    pub async fn connect(&mut self) -> Result<(), KiwoomError> {
        if self.connected {
            return Ok(());
        }
        let (socket, _) = connect_async(self.url.as_str())
            .await
            .map_err(|e| KiwoomError::Decode {
                raw: format!("ws connect error: {e}"),
            })?;
        self.socket = Some(socket);
        self.connected = true;
        Ok(())
    }

    pub async fn send_login_packet(&mut self, token: &str) -> Result<(), KiwoomError> {
        if !self.connected {
            self.connect().await?;
        }
        let payload = serde_json::json!({
            "trnm": "LOGIN",
            "token": token
        })
        .to_string();
        let socket = self.socket.as_mut().ok_or_else(|| KiwoomError::Decode {
            raw: "ws not connected".to_owned(),
        })?;

        socket
            .send(Message::Text(payload.into()))
            .await
            .map_err(|e| KiwoomError::Decode {
                raw: format!("ws send login error: {e}"),
            })?;
        Ok(())
    }

    pub async fn send_json(&mut self, payload: &Value) -> Result<(), KiwoomError> {
        if !self.connected {
            self.connect().await?;
        }

        let socket = self.socket.as_mut().ok_or_else(|| KiwoomError::Decode {
            raw: "ws not connected".to_owned(),
        })?;

        socket
            .send(Message::Text(payload.to_string().into()))
            .await
            .map_err(|e| KiwoomError::Decode {
                raw: format!("ws send json error: {e}"),
            })?;

        Ok(())
    }

    pub async fn recv_loop(&mut self) -> Result<serde_json::Value, KiwoomError> {
        if !self.connected {
            return Err(KiwoomError::Decode {
                raw: "ws not connected".to_owned(),
            });
        }

        loop {
            let next_msg = {
                let socket = self.socket.as_mut().ok_or_else(|| KiwoomError::Decode {
                    raw: "ws socket missing".to_owned(),
                })?;
                socket.next().await
            };

            //------
            match next_msg {
                Some(Ok(Message::Text(text))) => {
                    let raw = text.to_string();
                    let v = serde_json::from_str::<serde_json::Value>(&raw).map_err(|_| KiwoomError::Decode { raw })?;

                    if let Some(tr) = v.get("trnm").and_then(serde_json::Value::as_str) {
                        if tr != "PING" {
                            ts_dbg!(&v); // do not delete
                        }
                    }
                    return Ok(v);
                }
                Some(Ok(Message::Binary(bin))) => {
                    // let v = serde_json::from_slice::<serde_json::Value>(&bin).map_err(|_| KiwoomError::Decode {
                    //     raw: format!("non-json binary frame(len={})", bin.len()),
                    // })?;
                    // return Ok(v);
                    // binary는 포맷상 존재할 수 없으므로 에러 처리
                    return Err(KiwoomError::Decode {
                        raw: format!("unexpected binary frame(len={})", bin.len()),
                    });
                }
                Some(Ok(Message::Ping(payload))) => {
                    let socket = self.socket.as_mut().ok_or_else(|| KiwoomError::Decode {
                        raw: "ws socket missing".to_owned(),
                    })?;
                    socket
                        .send(Message::Pong(payload))
                        .await
                        .map_err(|e| KiwoomError::Decode {
                            raw: format!("ws pong send error: {e}"),
                        })?;
                }
                Some(Ok(Message::Pong(_))) => {
                    continue;
                }
                Some(Ok(Message::Close(frame))) => {
                    self.connected = false;
                    let _ = self.socket.take();
                    return Err(KiwoomError::Decode {
                        raw: format!("ws closed by server: {frame:?}"),
                    });
                }
                Some(Ok(Message::Frame(_))) => {
                    continue;
                }
                Some(Err(e)) => {
                    self.connected = false;
                    let _ = self.socket.take();
                    return Err(KiwoomError::Decode {
                        raw: format!("ws recv error: {e}"),
                    });
                }
                None => {
                    self.connected = false;
                    let _ = self.socket.take();
                    return Err(KiwoomError::Decode {
                        raw: "ws stream ended".to_owned(),
                    });
                }
            }
        }
    }

    pub async fn close(&mut self) -> Result<(), KiwoomError> {
        if let Some(mut socket) = self.socket.take() {
            socket.close(None).await.map_err(|e| KiwoomError::Decode {
                raw: format!("ws close error: {e}"),
            })?;
        }
        self.connected = false;
        Ok(())
    }

    pub async fn register(&mut self, data: Vec<WsRegData>) -> Result<(), KiwoomError> {
        if !self.connected {
            self.connect().await?;
        }
        let payload = serde_json::json!({
            "trnm": "REG",
            "grp_no": "1",
            "refresh": "1",
            "data": data,
        });

        let socket = self.socket.as_mut().ok_or_else(|| KiwoomError::Decode {
            raw: "ws not connected".to_owned(),
        })?;
        socket
            .send(Message::Text(payload.to_string().into()))
            .await
            .map_err(|e| KiwoomError::Decode {
                raw: format!("ws send REG error: {e}"),
            })?;
        Ok(())
    }
    pub async fn unregister(&mut self, data: Vec<WsRegData>) -> Result<(), KiwoomError> {
        if !self.connected {
            self.connect().await?;
        }

        let payload = serde_json::json!({
            "trnm": "REMOVE",
            "grp_no": "1",
            "data": data,
        });

        // send it
        let socket = self.socket.as_mut().ok_or_else(|| KiwoomError::Decode {
            raw: "ws not connected".to_owned(),
        })?;
        socket
            .send(Message::Text(payload.to_string().into()))
            .await
            .map_err(|e| KiwoomError::Decode {
                raw: format!("ws send REMOVE error: {e}"),
            })?;
        Ok(())
    }

    // --------------------------------
    // private
    // --------------------------------
}

pub mod ws_type {
    pub const 주식체결: &str = "0B";
    pub const 주식호가잔량: &str = "0D";
    pub const 장시작시간: &str = "0s";
}
