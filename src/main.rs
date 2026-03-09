#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")] // hide console window on Windows in release

mod api;
mod app;
mod constants;
mod tasks;
mod theme;
mod widgets;

use std::{collections::HashSet, sync::Arc};

use eframe::egui;

use crate::{
    api::kiwoom::{
        http::KiwoomApi,
        ws::{WsRegData, ws_type},
    },
    app::{MasterData, MyApp},
    constants::{CONTROL_PANEL_HEIGHT, CONTROL_PANEL_WIDTH},
    tasks::{
        rest_task::spawn_rest_task,
        ws_task::{WsCommand, spawn_ws_task},
    },
    theme::{configure_fonts, configure_sharp_style},
};

async fn fetch_master_all_pages(api: &KiwoomApi, mrkt_tp: &str) -> Result<Vec<serde_json::Value>, String> {
    let mut pages = Vec::new();
    let mut cont_yn: Option<String> = None;
    let mut next_key: Option<String> = None;

    loop {
        let page = api
            .fetch_master_stock(mrkt_tp, cont_yn.as_deref(), next_key.as_deref())
            .await
            .map_err(|e| format!("fetch_master_stock failed(mrkt_tp={mrkt_tp}): {e}"))?;

        pages.push(page.body);

        let should_continue = page
            .cont_yn
            .as_deref()
            .map(|v| v.trim().eq_ignore_ascii_case("Y"))
            .unwrap_or(false)
            && page.next_key.is_some();

        if !should_continue {
            break;
        }

        cont_yn = page.cont_yn;
        next_key = page.next_key;
    }

    Ok(pages)
}

fn count_master_records(pages: &[serde_json::Value]) -> usize {
    let mut seen = HashSet::new();
    for page in pages {
        collect_codes(page, &mut seen);
    }
    seen.len()
}

fn collect_codes(v: &serde_json::Value, seen: &mut HashSet<String>) {
    match v {
        serde_json::Value::Object(map) => {
            let code = ["code", "isu_cd", "stk_cd", "shrn_iscd", "isu_srt_cd"]
                .iter()
                .find_map(|k| map.get(*k).and_then(|x| x.as_str()))
                .map(str::trim)
                .filter(|s| !s.is_empty());

            if let Some(code) = code {
                seen.insert(code.to_owned());
            }

            for child in map.values() {
                collect_codes(child, seen);
            }
        }
        serde_json::Value::Array(arr) => {
            for child in arr {
                collect_codes(child, seen);
            }
        }
        _ => {}
    }
}

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

    let master = Arc::new(
        rt.block_on(async {
            let kospi_pages = fetch_master_all_pages(api.as_ref(), "0").await?; // kospi
            let kosdaq_pages = fetch_master_all_pages(api.as_ref(), "10").await?; // kosdaq
            Ok::<MasterData, String>(MasterData {
                kospi_pages,
                kosdaq_pages,
            })
        })
        .expect("master fetch failed"),
    );

    let kospi_count = count_master_records(&master.kospi_pages);
    let kosdaq_count = count_master_records(&master.kosdaq_pages);
    println!(
        "[MASTER] records: kospi={}, kosdaq={}, total={} (pages: kospi={}, kosdaq={})",
        kospi_count,
        kosdaq_count,
        kospi_count + kosdaq_count,
        master.kospi_pages.len(),
        master.kosdaq_pages.len()
    );

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
                Arc::clone(&master),
            )))
        }),
    )
}
