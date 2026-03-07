use std::collections::VecDeque;
use std::sync::Arc;

use serde_json::Value;
use tokio::sync::mpsc;

use crate::api::kiwoom::{
    http::KiwoomApi,
    ws::{KiwoomWs, WsRegData},
};

#[derive(Debug)]
pub enum WsCommand {
    Register(Vec<WsRegData>),
    Unregister(Vec<WsRegData>),
    Shutdown,
}

#[derive(Debug)]
pub enum WsEvent {
    Connected,
    LoginAck { ok: bool, raw: Value },
    Raw(Value),
    Error(String),
    Disconnected,
}

pub struct WsTaskChannels {
    pub from_ui_cmd_tx: mpsc::UnboundedSender<WsCommand>,
    pub from_ws_data_rx: mpsc::UnboundedReceiver<WsEvent>,
}

pub fn spawn_ws_task(rt: &Arc<tokio::runtime::Runtime>, api: Arc<KiwoomApi>) -> WsTaskChannels {
    let (from_ui_cmd_tx, from_ui_cmd_rx) = mpsc::unbounded_channel::<WsCommand>();
    let (from_ws_data_tx, from_ws_data_rx) = mpsc::unbounded_channel::<WsEvent>();

    rt.spawn(run_ws_worker(api, from_ui_cmd_rx, from_ws_data_tx));

    WsTaskChannels {
        from_ui_cmd_tx,
        from_ws_data_rx,
    }
}

async fn run_ws_worker(
    api: Arc<KiwoomApi>,
    mut from_ui_cmd_rx: mpsc::UnboundedReceiver<WsCommand>,
    from_ws_data_tx: mpsc::UnboundedSender<WsEvent>,
) {
    enum LoginState {
        Pending,
        Authenticated,
    }

    let mut ws = KiwoomWs::new();
    let mut login_state = LoginState::Pending;
    let mut pending_cmds: VecDeque<WsCommand> = VecDeque::new();

    if let Err(e) = ws.connect().await {
        let _ = from_ws_data_tx.send(WsEvent::Error(format!("ws connect failed: {e}")));
        let _ = from_ws_data_tx.send(WsEvent::Disconnected);
        return;
    }
    let _ = from_ws_data_tx.send(WsEvent::Connected);

    let token = match api.access_token().await {
        Ok(token) => token,
        Err(e) => {
            let _ = from_ws_data_tx.send(WsEvent::Error(format!("token fetch failed: {e}")));
            let _ = from_ws_data_tx.send(WsEvent::Disconnected);
            return;
        }
    };

    if let Err(e) = ws.send_login_packet(&token).await {
        let _ = from_ws_data_tx.send(WsEvent::Error(format!("login send failed: {e}")));
        let _ = from_ws_data_tx.send(WsEvent::Disconnected);
        return;
    }

    loop {
        tokio::select! {
            cmd = from_ui_cmd_rx.recv() => {
                match cmd {
                    Some(WsCommand::Register(data)) => {
                        match login_state {
                            LoginState::Pending => pending_cmds.push_back(WsCommand::Register(data)),
                            LoginState::Authenticated => {
                                if let Err(e) = ws.register(data).await {
                                    let _ = from_ws_data_tx.send(WsEvent::Error(format!("register failed: {e}")));
                                }
                            }
                        }
                    }
                    Some(WsCommand::Unregister(data)) => {
                        match login_state {
                            LoginState::Pending => pending_cmds.push_back(WsCommand::Unregister(data)),
                            LoginState::Authenticated => {
                                if let Err(e) = ws.unregister(data).await {
                                    let _ = from_ws_data_tx.send(WsEvent::Error(format!("unregister failed: {e}")));
                                }
                            }
                        }
                    }
                    Some(WsCommand::Shutdown) | None => {
                        let _ = ws.close().await;
                        let _ = from_ws_data_tx.send(WsEvent::Disconnected);
                        break;
                    }
                }
            }
            incoming = ws.recv_loop() => {
                match incoming {
                    Ok(v) => {
                        if is_ping(&v) {
                            if let Err(e) = ws.send_json(&v).await {
                                let _ = from_ws_data_tx.send(WsEvent::Error(format!("ping echo failed: {e}")));
                                let _ = from_ws_data_tx.send(WsEvent::Disconnected);
                                break;
                            }
                            continue;
                        }

                        if is_login_ack(&v) {
                            let ok = return_code_is_ok(&v);
                            if ok {
                                login_state = LoginState::Authenticated;
                                while let Some(pending) = pending_cmds.pop_front() {
                                    let send_result = match pending {
                                        WsCommand::Register(data) => ws.register(data).await,
                                        WsCommand::Unregister(data) => ws.unregister(data).await,
                                        WsCommand::Shutdown => {
                                            let _ = ws.close().await;
                                            let _ = from_ws_data_tx.send(WsEvent::Disconnected);
                                            return;
                                        }
                                    };

                                    if let Err(e) = send_result {
                                        let _ = from_ws_data_tx.send(WsEvent::Error(format!("pending command failed: {e}")));
                                    }
                                }
                            }
                            let _ = from_ws_data_tx.send(WsEvent::LoginAck { ok, raw: v });
                        } else {
                            let _ = from_ws_data_tx.send(WsEvent::Raw(v));
                        }
                    }
                    Err(e) => {
                        let _ = from_ws_data_tx.send(WsEvent::Error(format!("recv failed: {e}")));
                        let _ = from_ws_data_tx.send(WsEvent::Disconnected);
                        break;
                    }
                }
            }
        }
    }
}

fn is_ping(v: &Value) -> bool {
    v.get("trnm").and_then(Value::as_str) == Some("PING")
}

fn is_login_ack(v: &Value) -> bool {
    v.get("trnm").and_then(Value::as_str) == Some("LOGIN")
}

fn return_code_is_ok(v: &Value) -> bool {
    match v.get("return_code") {
        Some(Value::Number(n)) => n.as_i64() == Some(0),
        Some(Value::String(s)) => s == "0",
        _ => false,
    }
}
