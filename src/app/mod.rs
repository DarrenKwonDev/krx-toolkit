use std::sync::{
    Arc,
    atomic::{AtomicBool, Ordering},
};

use eframe::egui;

mod control_panel;
mod viewport;

pub struct MyApp {
    show_confirmation_dialog: Arc<AtomicBool>,
    allowed_to_close: Arc<AtomicBool>,
    always_on_top: bool,
    opened_viewports: Vec<egui::ViewportId>,
    // ----viewport 상태 변수----
    show_settings_viewport: Arc<AtomicBool>,
    show_account_viewport: Arc<AtomicBool>,
    order_tool_viewports: Vec<(egui::ViewportId, Arc<AtomicBool>, u64)>,
    next_order_tool_seq: u64,
}

impl Default for MyApp {
    fn default() -> Self {
        Self {
            show_confirmation_dialog: Default::default(),
            allowed_to_close: Default::default(),
            always_on_top: Default::default(),
            opened_viewports: vec![egui::ViewportId::ROOT],
            // ----viewport 상태 변수----
            show_settings_viewport: Default::default(),
            show_account_viewport: Default::default(),
            order_tool_viewports: vec![],
            next_order_tool_seq: 0,
        }
    }
}

impl eframe::App for MyApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        if ctx.input(|i| i.viewport().close_requested()) && !self.allowed_to_close.load(Ordering::Relaxed) {
            ctx.send_viewport_cmd(egui::ViewportCommand::CancelClose);
            self.show_confirmation_dialog.store(true, Ordering::Relaxed);
        }

        // main control panel
        self.render_control_panel(ctx);

        // setting viewport
        self.render_settings_viewport(ctx);
        self.render_account_viewport(ctx);
        self.render_order_tool_viewport(ctx);

        self.render_exit_confirm_viewport(ctx);
    }
}
