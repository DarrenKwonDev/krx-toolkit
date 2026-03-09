#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")] // hide console window on Windows in release

mod api;
mod app;
mod constants;
mod tasks;
mod theme;

use std::sync::Arc;

use eframe::egui;

use crate::{
    api::kiwoom::{
        http::KiwoomApi,
        ws::{WsRegData, ws_type},
    },
    app::MyApp,
    constants::{CONTROL_PANEL_HEIGHT, CONTROL_PANEL_WIDTH},
    tasks::{
        rest_task::spawn_rest_task,
        ws_task::{WsCommand, spawn_ws_task},
    },
    theme::{configure_fonts, configure_sharp_style},
};

fn bootstrap() {
    dotenvy::dotenv().ok();

    let rust_log = std::env::var("RUST_LOG").unwrap_or_else(|_| "info".to_owned());

    // SAFETY: we call this from the main thread without any other threads running.
    #[expect(unsafe_code)]
    unsafe {
        std::env::set_var("RUST_LOG", rust_log);
    };
    env_logger::init();
}

fn main() -> eframe::Result {
    bootstrap();

    let rt = Arc::new(tokio::runtime::Runtime::new().expect("tokio runtime create failed"));
    let api = Arc::new(rt.block_on(KiwoomApi::new()).expect("kiwoom api init failed"));

    let ws_channels = spawn_ws_task(&rt, Arc::clone(&api));
    let rest_channels = spawn_rest_task(&rt, Arc::clone(&api));

    // initial register
    let _ = ws_channels.from_ui_cmd_tx.send(WsCommand::Register(vec![
        WsRegData {
            item: vec!["".to_owned()],
            r#type: vec![ws_type::장시작시간.to_owned()],
        },
        // WsRegData {
        //     item: vec!["005930".to_owned()],
        //     r#type: vec![ws_type::주식체결.to_owned()],
        // },
        // WsRegData {
        //     item: vec!["005930".to_owned()],
        //     r#type: vec![ws_type::주식호가잔량.to_owned()],
        // },
    ]));

    // render eframe
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_title("krx-toolkit")
            .with_inner_size([CONTROL_PANEL_WIDTH, CONTROL_PANEL_HEIGHT])
            .with_resizable(false),
        ..Default::default()
    };
    eframe::run_native(
        "krx-toolkit",
        options,
        Box::new(|cc| {
            configure_fonts(&cc.egui_ctx);
            configure_sharp_style(&cc.egui_ctx);
            Ok(Box::new(MyApp::new(
                ws_channels.from_ui_cmd_tx,
                ws_channels.from_ws_data_rx,
                rest_channels.from_ui_rest_cmd_tx,
                rest_channels.from_rest_data_rx,
            )))
        }),
    )
}
