use std::collections::{HashMap, HashSet, VecDeque};
use std::sync::Arc;

use serde_json::Value;
use tokio::sync::mpsc;

use crate::api::kiwoom::{
    http::KiwoomApi,
    ws::{KiwoomWs, WsRegData},
};

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct WsTopic {
    pub item: String,
    pub ty: String,
}

#[derive(Debug)]
pub enum WsCommand {
    Register(Vec<WsRegData>),
    Unregister(Vec<WsRegData>),
    Subscribe { subscriber_id: u64, topics: Vec<WsTopic> },
    Unsubscribe { subscriber_id: u64, topics: Vec<WsTopic> },
    UnsubscribeAll { subscriber_id: u64 },
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

    // egui에서는 cmd는 전송하고, data는 받아야 하므로
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
    let mut topic_subscribers: HashMap<WsTopic, HashSet<u64>> = HashMap::new();
    let mut subscriber_topics: HashMap<u64, HashSet<WsTopic>> = HashMap::new();

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
                    Some(WsCommand::Subscribe {
                        subscriber_id,
                        topics,
                    }) => {
                        match login_state {
                            LoginState::Pending => pending_cmds.push_back(WsCommand::Subscribe {
                                subscriber_id,
                                topics,
                            }),
                            LoginState::Authenticated => {
                                let to_register = apply_subscribe(
                                    &mut topic_subscribers,
                                    &mut subscriber_topics,
                                    subscriber_id,
                                    topics,
                                );
                                let reg_data = ws_topics_to_reg_data(to_register);
                                if !reg_data.is_empty() {
                                    if let Err(e) = ws.register(reg_data).await {
                                        let _ = from_ws_data_tx.send(WsEvent::Error(format!("subscribe failed: {e}")));
                                    }
                                }
                            }
                        }
                    }
                    Some(WsCommand::Unsubscribe {
                        subscriber_id,
                        topics,
                    }) => {
                        match login_state {
                            LoginState::Pending => pending_cmds.push_back(WsCommand::Unsubscribe {
                                subscriber_id,
                                topics,
                            }),
                            LoginState::Authenticated => {
                                let to_unregister = apply_unsubscribe(
                                    &mut topic_subscribers,
                                    &mut subscriber_topics,
                                    subscriber_id,
                                    topics,
                                );
                                let unreg_data = ws_topics_to_reg_data(to_unregister);
                                if !unreg_data.is_empty() {
                                    if let Err(e) = ws.unregister(unreg_data).await {
                                        let _ = from_ws_data_tx.send(WsEvent::Error(format!("unsubscribe failed: {e}")));
                                    }
                                }
                            }
                        }
                    }
                    Some(WsCommand::UnsubscribeAll { subscriber_id }) => {
                        match login_state {
                            LoginState::Pending => {
                                pending_cmds.push_back(WsCommand::UnsubscribeAll { subscriber_id })
                            }
                            LoginState::Authenticated => {
                                let to_unregister = apply_unsubscribe_all(
                                    &mut topic_subscribers,
                                    &mut subscriber_topics,
                                    subscriber_id,
                                );
                                let unreg_data = ws_topics_to_reg_data(to_unregister);
                                if !unreg_data.is_empty() {
                                    if let Err(e) = ws.unregister(unreg_data).await {
                                        let _ = from_ws_data_tx.send(WsEvent::Error(format!("unsubscribe_all failed: {e}")));
                                    }
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
                                        WsCommand::Subscribe {
                                            subscriber_id,
                                            topics,
                                        } => {
                                            let to_register = apply_subscribe(
                                                &mut topic_subscribers,
                                                &mut subscriber_topics,
                                                subscriber_id,
                                                topics,
                                            );
                                            let reg_data = ws_topics_to_reg_data(to_register);
                                            if reg_data.is_empty() {
                                                Ok(())
                                            } else {
                                                ws.register(reg_data).await
                                            }
                                        }
                                        WsCommand::Unsubscribe {
                                            subscriber_id,
                                            topics,
                                        } => {
                                            let to_unregister = apply_unsubscribe(
                                                &mut topic_subscribers,
                                                &mut subscriber_topics,
                                                subscriber_id,
                                                topics,
                                            );
                                            let unreg_data = ws_topics_to_reg_data(to_unregister);
                                            if unreg_data.is_empty() {
                                                Ok(())
                                            } else {
                                                ws.unregister(unreg_data).await
                                            }
                                        }
                                        WsCommand::UnsubscribeAll { subscriber_id } => {
                                            let to_unregister = apply_unsubscribe_all(
                                                &mut topic_subscribers,
                                                &mut subscriber_topics,
                                                subscriber_id,
                                            );
                                            let unreg_data = ws_topics_to_reg_data(to_unregister);
                                            if unreg_data.is_empty() {
                                                Ok(())
                                            } else {
                                                ws.unregister(unreg_data).await
                                            }
                                        }
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

pub fn extract_trnm(v: &Value) -> Option<&str> {
    v.get("trnm").and_then(Value::as_str)
}

pub fn extract_ws_topic(v: &Value) -> Option<WsTopic> {
    let ty = pick_first_str(v, &["type", "real_type", "tr_type", "ty", "trnm"])?;
    let item = pick_first_str(v, &["item", "code", "stk_cd", "isu_cd", "symbol", "shrn_iscd"])?;

    let ty = ty.trim();
    let item = item.trim();
    if ty.is_empty() || item.is_empty() {
        return None;
    }

    Some(WsTopic {
        item: item.to_owned(),
        ty: ty.to_owned(),
    })
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

fn pick_first_str<'a>(v: &'a Value, keys: &[&str]) -> Option<&'a str> {
    keys.iter().find_map(|key| v.get(*key).and_then(Value::as_str))
}

fn apply_subscribe(
    topic_subscribers: &mut HashMap<WsTopic, HashSet<u64>>,
    subscriber_topics: &mut HashMap<u64, HashSet<WsTopic>>,
    subscriber_id: u64,
    topics: Vec<WsTopic>,
) -> Vec<WsTopic> {
    let unique_topics: HashSet<WsTopic> = topics.into_iter().collect();
    let subscriber_set = subscriber_topics.entry(subscriber_id).or_default();
    let mut to_register = Vec::new();

    for topic in unique_topics {
        if !subscriber_set.insert(topic.clone()) {
            continue;
        }

        let subscribers = topic_subscribers.entry(topic.clone()).or_default();
        let was_empty = subscribers.is_empty();
        subscribers.insert(subscriber_id);
        if was_empty {
            to_register.push(topic);
        }
    }

    to_register
}

fn apply_unsubscribe(
    topic_subscribers: &mut HashMap<WsTopic, HashSet<u64>>,
    subscriber_topics: &mut HashMap<u64, HashSet<WsTopic>>,
    subscriber_id: u64,
    topics: Vec<WsTopic>,
) -> Vec<WsTopic> {
    let unique_topics: HashSet<WsTopic> = topics.into_iter().collect();
    let mut to_unregister = Vec::new();

    let Some(subscriber_set) = subscriber_topics.get_mut(&subscriber_id) else {
        return to_unregister;
    };

    for topic in unique_topics {
        if !subscriber_set.remove(&topic) {
            continue;
        }

        if let Some(subscribers) = topic_subscribers.get_mut(&topic) {
            subscribers.remove(&subscriber_id);
            if subscribers.is_empty() {
                topic_subscribers.remove(&topic);
                to_unregister.push(topic);
            }
        }
    }

    if subscriber_set.is_empty() {
        subscriber_topics.remove(&subscriber_id);
    }

    to_unregister
}

fn apply_unsubscribe_all(
    topic_subscribers: &mut HashMap<WsTopic, HashSet<u64>>,
    subscriber_topics: &mut HashMap<u64, HashSet<WsTopic>>,
    subscriber_id: u64,
) -> Vec<WsTopic> {
    let Some(subscriber_set) = subscriber_topics.remove(&subscriber_id) else {
        return Vec::new();
    };

    let mut to_unregister = Vec::new();
    for topic in subscriber_set {
        if let Some(subscribers) = topic_subscribers.get_mut(&topic) {
            subscribers.remove(&subscriber_id);
            if subscribers.is_empty() {
                topic_subscribers.remove(&topic);
                to_unregister.push(topic);
            }
        }
    }

    to_unregister
}

fn ws_topics_to_reg_data(topics: Vec<WsTopic>) -> Vec<WsRegData> {
    let mut grouped: HashMap<String, HashSet<String>> = HashMap::new();

    for topic in topics {
        grouped.entry(topic.ty).or_default().insert(topic.item);
    }

    let mut grouped_vec = grouped.into_iter().collect::<Vec<_>>();
    grouped_vec.sort_by(|(lhs_ty, _), (rhs_ty, _)| lhs_ty.cmp(rhs_ty));

    grouped_vec
        .into_iter()
        .map(|(ty, items)| {
            let mut item = items.into_iter().collect::<Vec<_>>();
            item.sort();
            WsRegData { item, r#type: vec![ty] }
        })
        .collect()
}
