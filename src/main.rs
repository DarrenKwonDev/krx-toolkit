#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")] // hide console window on Windows in release

mod constants;

use std::sync::{
    Arc,
    atomic::{AtomicBool, Ordering},
};

use eframe::egui::{self, FontData, FontDefinitions, FontFamily, Ui};

use crate::constants::{
    CONTROL_PANEL_BOTTOM_MARGIN, CONTROL_PANEL_BUTTON_H, CONTROL_PANEL_BUTTON_W, CONTROL_PANEL_HEIGHT,
    CONTROL_PANEL_SIDE_MAGIN, CONTROL_PANEL_WIDTH, SETTING_VIEWPORT_ID,
};

fn main() -> eframe::Result {
    let rust_log = std::env::var("RUST_LOG").unwrap_or_else(|_| "info".to_owned());

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

#[derive(Default)]
struct MyApp {
    show_settings_viewport: Arc<AtomicBool>,
    _show_confirmation_dialog: bool,
    _allowed_to_close: bool,
    always_on_top: bool,
}

impl eframe::App for MyApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default()
            .frame(
                egui::Frame::central_panel(ctx.style().as_ref())
                    .inner_margin(0)
                    .outer_margin(0)
                    .stroke(egui::Stroke::NONE),
            )
            .show(ctx, |ui| {
                let rect = ui.max_rect();
                let split_y = (rect.max.y - CONTROL_PANEL_BOTTOM_MARGIN).max(rect.min.y);

                // (좌상(x, y), 우하(x, y))
                let top_rect = egui::Rect::from_min_max(rect.min, egui::pos2(rect.max.x, split_y));
                let bottom_rect = egui::Rect::from_min_max(egui::pos2(rect.min.x, split_y), rect.max);

                let top_left_btn_size = egui::vec2(CONTROL_PANEL_BUTTON_W, CONTROL_PANEL_BUTTON_H);
                let top_right_btn_size = egui::vec2(CONTROL_PANEL_BUTTON_H, CONTROL_PANEL_BUTTON_H);

                ui.scope_builder(egui::UiBuilder::new().max_rect(top_rect), |ui| {
                    ui.with_layout(egui::Layout::left_to_right(egui::Align::Min), |ui| {
                        ui.spacing_mut().item_spacing.x = 1.0;

                        ui.add_space(CONTROL_PANEL_SIDE_MAGIN);
                        if ui.add_sized(top_left_btn_size, egui::Button::new("설정")).clicked() {
                            self.show_settings_viewport.store(true, Ordering::Relaxed);
                        };
                        ui.add_sized(top_left_btn_size, egui::Button::new("계좌관리"));
                        ui.add_sized(top_left_btn_size, egui::Button::new("주문도구"));
                        ui.add_sized(top_left_btn_size, egui::Button::new("주문체결"));
                        ui.add_sized(top_left_btn_size, egui::Button::new("빠른호가"));
                        ui.add_sized(top_left_btn_size, egui::Button::new("잔고손익"));

                        ui.add_space(ui.available_width());

                        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                            ui.add_space(CONTROL_PANEL_SIDE_MAGIN);

                            let pin_label = if self.always_on_top { "P*" } else { "P" };
                            if ui.add_sized(top_right_btn_size, egui::Button::new(pin_label)).clicked() {
                                self.always_on_top = !self.always_on_top;
                                let level = if self.always_on_top {
                                    egui::WindowLevel::AlwaysOnTop
                                } else {
                                    egui::WindowLevel::Normal
                                };
                                ctx.send_viewport_cmd(egui::ViewportCommand::WindowLevel(level));
                            }
                        });
                    });
                });

                ui.painter().hline(
                    rect.x_range(),
                    split_y,
                    egui::Stroke::new(1.0, ui.visuals().widgets.noninteractive.bg_stroke.color),
                );

                ui.scope_builder(egui::UiBuilder::new().max_rect(bottom_rect), |ui| {
                    ui.with_layout(egui::Layout::left_to_right(egui::Align::Center), |ui| {
                        ui.add_space(CONTROL_PANEL_SIDE_MAGIN);
                        ui.label("krx-tools ... bottom");
                    });
                });
            }); // end Central

        // setting viewport
        self.render_settings_viewport(ctx);
    }
}

impl MyApp {
    fn render_settings_viewport(&mut self, ctx: &egui::Context) {
        if self.show_settings_viewport.load(Ordering::Relaxed) {
            let show_settings = Arc::clone(&self.show_settings_viewport);

            // viewport
            ctx.show_viewport_deferred(
                egui::ViewportId::from_hash_of(SETTING_VIEWPORT_ID),
                egui::ViewportBuilder::default()
                    .with_title("설정")
                    .with_inner_size([360.0, 220.0]),
                move |child_ctx, class| {
                    // embedded는 허용하지 않지만 fallback으로 둔다
                    if class == egui::ViewportClass::Embedded {
                        egui::Window::new("설정").show(child_ctx, |ui| {
                            ui.label("settings placeholder");
                        });
                    } else {
                        egui::CentralPanel::default().show(child_ctx, |ui| {
                            _debug_check_rect(ui);
                        });
                    }

                    // 닫힐 경우 상태 변수 false로 재지정
                    if child_ctx.input(|i| i.viewport().close_requested()) {
                        show_settings.store(false, Ordering::Relaxed);
                    }
                },
            );
        }
    }
}

// ---------------------------
fn configure_fonts(ctx: &egui::Context) {
    let mut fonts = FontDefinitions::default();
    fonts.font_data.insert(
        "korean".to_owned(),
        FontData::from_static(include_bytes!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/assets/fonts/PretendardVariable.ttf"
        )))
        .into(),
    );
    fonts
        .families
        .entry(FontFamily::Proportional)
        .or_default()
        .insert(0, "korean".to_owned());
    fonts
        .families
        .entry(FontFamily::Monospace)
        .or_default()
        .insert(0, "korean".to_owned());
    ctx.set_fonts(fonts);
}
fn configure_sharp_style(ctx: &egui::Context) {
    ctx.all_styles_mut(|style| {
        // 버튼/체크박스/슬라이더 등 위젯 모서리
        style.visuals.widgets.noninteractive.corner_radius = egui::CornerRadius::ZERO;
        style.visuals.widgets.inactive.corner_radius = egui::CornerRadius::ZERO;
        style.visuals.widgets.hovered.corner_radius = egui::CornerRadius::ZERO;
        style.visuals.widgets.active.corner_radius = egui::CornerRadius::ZERO;
        style.visuals.widgets.open.corner_radius = egui::CornerRadius::ZERO;
        // 창/메뉴 자체 모서리도 각지게
        style.visuals.window_corner_radius = egui::CornerRadius::ZERO;
        style.visuals.menu_corner_radius = egui::CornerRadius::ZERO;
    });
}
fn _debug_check_rect(ui: &Ui) {
    let panel_rect = ui.max_rect();
    let content_rect = ui.available_rect_before_wrap();
    ui.painter().rect_stroke(
        panel_rect,
        0.0,
        egui::Stroke::new(1.0, egui::Color32::RED),
        egui::StrokeKind::Middle,
    );
    ui.painter().rect_stroke(
        content_rect,
        0.0,
        egui::Stroke::new(1.0, egui::Color32::GREEN),
        egui::StrokeKind::Middle,
    );
}
