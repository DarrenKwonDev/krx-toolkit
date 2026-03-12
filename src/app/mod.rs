use std::{
    collections::{HashMap, HashSet},
    sync::{
        Arc,
        atomic::{AtomicBool, Ordering},
    },
};

use eframe::egui;
use tokio::sync::mpsc;

use crate::tasks::{
    rest_task::{RestCommand, RestEvent},
    ws_task::{WsCommand, WsEvent, WsTopic, extract_trnm, extract_ws_topic},
};

mod control_panel;
mod order_normal;
mod viewport;

#[derive(Debug, Clone)]
pub struct MasterData {
    pub kospi_pages: Vec<serde_json::Value>,
    pub kosdaq_pages: Vec<serde_json::Value>,
}

pub struct MyApp {
    show_confirmation_dialog: Arc<AtomicBool>,
    allowed_to_close: Arc<AtomicBool>,
    always_on_top: bool,
    opened_viewports: Vec<egui::ViewportId>,
    // ----channel----
    ws_cmd_tx: mpsc::UnboundedSender<WsCommand>,
    ws_data_rx: mpsc::UnboundedReceiver<WsEvent>,
    rest_cmd_tx: mpsc::UnboundedSender<RestCommand>,
    rest_data_rx: mpsc::UnboundedReceiver<RestEvent>,
    master: Arc<MasterData>,
    // ----viewport 상태 변수----
    show_settings_viewport: Arc<AtomicBool>,
    show_account_viewport: Arc<AtomicBool>,
    show_emergency_order_viewport: Arc<AtomicBool>,
    order_tool_viewports: Vec<(egui::ViewportId, Arc<AtomicBool>, u64)>,
    order_active_topics: HashMap<u64, HashSet<WsTopic>>,
    order_selected_codes: HashMap<u64, String>,
    next_order_tool_seq: u64,
    // ----하단 바 상태----
    ws_connected: bool,
    ws_login_ok: bool,
}

impl MyApp {
    pub fn new(
        ws_cmd_tx: mpsc::UnboundedSender<WsCommand>,
        ws_data_rx: mpsc::UnboundedReceiver<WsEvent>,
        rest_cmd_tx: mpsc::UnboundedSender<RestCommand>,
        rest_data_rx: mpsc::UnboundedReceiver<RestEvent>,
        master: Arc<MasterData>,
    ) -> Self {
        Self {
            show_confirmation_dialog: Default::default(),
            allowed_to_close: Default::default(),
            always_on_top: Default::default(),
            opened_viewports: vec![egui::ViewportId::ROOT],
            // ----channel----
            ws_cmd_tx,
            ws_data_rx,
            rest_cmd_tx,
            rest_data_rx,
            master,
            // ----viewport 상태 변수----
            show_settings_viewport: Default::default(),
            show_account_viewport: Default::default(),
            show_emergency_order_viewport: Default::default(),
            order_tool_viewports: vec![],
            order_active_topics: HashMap::new(),
            order_selected_codes: HashMap::new(),
            next_order_tool_seq: 0,
            ws_connected: false,
            ws_login_ok: false,
        }
    }

    pub(super) fn ws_unsubscribe_all(&mut self, subscriber_id: u64) {
        let _ = self.ws_cmd_tx.send(WsCommand::UnsubscribeAll { subscriber_id });
        self.order_active_topics.remove(&subscriber_id);
        self.order_selected_codes.remove(&subscriber_id);
    }

    fn poll_background_events(&mut self) {
        while let Ok(evt) = self.ws_data_rx.try_recv() {
            match evt {
                WsEvent::Connected => {
                    self.ws_connected = true;
                }
                WsEvent::LoginAck { ok, .. } => {
                    self.ws_login_ok = ok;
                }
                WsEvent::Raw(raw) => {
                    let _trnm = extract_trnm(&raw);
                    let _routing_targets = extract_ws_topic(&raw)
                        .map(|topic| self.subscribers_for_topic(&topic))
                        .unwrap_or_default();
                }
                WsEvent::Error(_) => {
                    self.ws_connected = false;
                    self.ws_login_ok = false;
                }
                WsEvent::Disconnected => {
                    self.ws_connected = false;
                    self.ws_login_ok = false;
                }
            }
        }
        while let Ok(_evt) = self.rest_data_rx.try_recv() {
            // 지금은 하단바 요구사항이 WS 중심이니까 일단 비워둬도 됨
            // 이후 REST 상태 표시할 때 match 추가
        }
    }

    fn subscribers_for_topic(&self, topic: &WsTopic) -> Vec<u64> {
        self.order_active_topics
            .iter()
            .filter_map(|(seq, topics)| topics.contains(topic).then_some(*seq))
            .collect()
    }
}

impl eframe::App for MyApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        if ctx.input(|i| i.viewport().close_requested())
            && !self.allowed_to_close.load(Ordering::Relaxed)
            && !self.show_confirmation_dialog.load(Ordering::Relaxed)
        {
            ctx.send_viewport_cmd(egui::ViewportCommand::CancelClose);
            self.show_confirmation_dialog.store(true, Ordering::Relaxed);
        }

        self.poll_background_events();

        // main control panel
        self.render_control_panel(ctx);

        // setting viewport
        self.render_settings_viewport(ctx);
        self.render_account_viewport(ctx);
        self.render_emergency_order_viewport(ctx);
        self.render_order_tool_viewport(ctx);

        self.render_exit_confirm_viewport(ctx);
    }
}
