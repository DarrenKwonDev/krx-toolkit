#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")] // hide console window on Windows in release

mod app;
mod constants;
mod theme;

use eframe::egui;

use crate::{
    app::MyApp,
    constants::{CONTROL_PANEL_HEIGHT, CONTROL_PANEL_WIDTH},
    theme::{configure_fonts, configure_sharp_style},
};

fn main() -> eframe::Result {
    dotenvy::dotenv().ok();

    let rust_log = std::env::var("RUST_LOG").unwrap_or_else(|_| "info".to_owned());
    let app_key = std::env::var("KIWOOM_APP_KEY").expect("app key missing in .env");
    let secret_key = std::env::var("KIWOOM_SECRET_KEY").expect("secret_key missing in .env");

    // SAFETY: we call this from the main thread without any other threads running.
    #[expect(unsafe_code)]
    unsafe {
        std::env::set_var("RUST_LOG", rust_log);
    };
    env_logger::init();

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
            Ok(Box::<MyApp>::default())
        }),
    )
}
