use std::sync::Arc;

use tokio::sync::mpsc;

use crate::api::kiwoom::http::KiwoomApi;

#[derive(Debug, Clone)]
pub enum RestCommand {
    AccessToken { request_id: u64 },
    Shutdown,
}

#[derive(Debug, Clone)]
pub enum RestEvent {
    AccessToken { request_id: u64, token: String },
    Error { request_id: Option<u64>, message: String },
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
            RestCommand::Shutdown => {
                let _ = from_rest_data_tx.send(RestEvent::Stopped);
                break;
            }
        }
    }
}
