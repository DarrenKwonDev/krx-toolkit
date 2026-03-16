use std::sync::Arc;

use serde_json::Value;
use tokio::sync::mpsc;

use crate::api::kiwoom::http::{BuyStockRequest, KiwoomApi};

#[derive(Debug, Clone)]
pub enum RestCommand {
    AccessToken {
        request_id: u64,
    },
    FetchMasterStock {
        request_id: u64,
        mrkt_tp: String,
    },
    BuyStock {
        request_id: u64,
        dmst_stex_tp: String,
        stk_cd: String,
        ord_qty: u64,
        ord_uv: Option<u64>,
        trde_tp: String,
        cond_uv: Option<u64>,
    },
    Shutdown,
}

#[derive(Debug, Clone)]
pub enum RestEvent {
    AccessToken {
        request_id: u64,
        token: String,
    },
    MasterStock {
        request_id: u64,
        pages: Vec<Value>,
        total_pages: usize,
    },
    BuyStock {
        request_id: u64,
        stk_cd: String,
        ord_qty: u64,
        trde_tp: String,
        ord_no: Option<String>,
        dmst_stex_tp: Option<String>,
    },
    Error {
        request_id: Option<u64>,
        message: String,
    },
    Stopped,
}

pub struct RestTaskChannels {
    pub from_ui_rest_cmd_tx: mpsc::UnboundedSender<RestCommand>,
    pub from_rest_data_rx: mpsc::UnboundedReceiver<RestEvent>,
}

pub fn spawn_rest_task(rt: &Arc<tokio::runtime::Runtime>, api: Arc<KiwoomApi>) -> RestTaskChannels {
    let (from_ui_rest_cmd_tx, from_ui_rest_cmd_rx) = mpsc::unbounded_channel::<RestCommand>();
    let (from_rest_data_tx, from_rest_data_rx) = mpsc::unbounded_channel::<RestEvent>();

    rt.spawn(run_rest_worker(api, from_ui_rest_cmd_rx, from_rest_data_tx));

    RestTaskChannels {
        from_ui_rest_cmd_tx,
        from_rest_data_rx,
    }
}

async fn run_rest_worker(
    api: Arc<KiwoomApi>,
    mut from_ui_rest_cmd_rx: mpsc::UnboundedReceiver<RestCommand>,
    from_rest_data_tx: mpsc::UnboundedSender<RestEvent>,
) {
    while let Some(cmd) = from_ui_rest_cmd_rx.recv().await {
        match cmd {
            RestCommand::AccessToken { request_id } => match api.access_token().await {
                Ok(token) => {
                    let _ = from_rest_data_tx.send(RestEvent::AccessToken { request_id, token });
                }
                Err(e) => {
                    let _ = from_rest_data_tx.send(RestEvent::Error {
                        request_id: Some(request_id),
                        message: format!("access_token failed: {e}"),
                    });
                }
            },
            RestCommand::FetchMasterStock { request_id, mrkt_tp } => {
                let mut pages: Vec<Value> = Vec::new();
                let mut cont_yn: Option<String> = None;
                let mut next_key: Option<String> = None;

                loop {
                    let page = match api
                        .fetch_master_stock(&mrkt_tp, cont_yn.as_deref(), next_key.as_deref())
                        .await
                    {
                        Ok(p) => p,
                        Err(e) => {
                            let _ = from_rest_data_tx.send(RestEvent::Error {
                                request_id: Some(request_id),
                                message: format!("fetch_master_stock failed: {e}"),
                            });
                            break;
                        }
                    };

                    pages.push(page.body);

                    let continue_next = should_continue(page.cont_yn.as_deref()) && page.next_key.is_some();
                    if !continue_next {
                        let total_pages = pages.len();
                        let _ = from_rest_data_tx.send(RestEvent::MasterStock {
                            request_id,
                            pages,
                            total_pages,
                        });
                        break;
                    }

                    cont_yn = page.cont_yn;
                    next_key = page.next_key;
                }
            }
            RestCommand::BuyStock {
                request_id,
                dmst_stex_tp,
                stk_cd,
                ord_qty,
                ord_uv,
                trde_tp,
                cond_uv,
            } => {
                let req = BuyStockRequest {
                    dmst_stex_tp,
                    stk_cd: stk_cd.clone(),
                    ord_qty,
                    ord_uv,
                    trde_tp: trde_tp.clone(),
                    cond_uv,
                };
                match api.buy_stock(req).await {
                    Ok(resp) => {
                        let _ = from_rest_data_tx.send(RestEvent::BuyStock {
                            request_id,
                            stk_cd,
                            ord_qty,
                            trde_tp,
                            ord_no: resp.ord_no,
                            dmst_stex_tp: resp.dmst_stex_tp,
                        });
                    }
                    Err(e) => {
                        let _ = from_rest_data_tx.send(RestEvent::Error {
                            request_id: Some(request_id),
                            message: format!("buy_stock failed: {e}"),
                        });
                    }
                }
            }
            RestCommand::Shutdown => {
                let _ = from_rest_data_tx.send(RestEvent::Stopped);
                break;
            }
        }
    }
}

fn should_continue(cont_yn: Option<&str>) -> bool {
    cont_yn.map(|v| v.trim().eq_ignore_ascii_case("Y")).unwrap_or(false)
}
