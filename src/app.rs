use std::sync::{
    Arc,
    atomic::{AtomicBool, Ordering},
};

use eframe::egui;

use crate::{
    constants::{
        ACCOUNT_VIEWPORT_H, ACCOUNT_VIEWPORT_ID, ACCOUNT_VIEWPORT_W, CONTROL_PANEL_BOTTOM_MARGIN,
        CONTROL_PANEL_BUTTON_H, CONTROL_PANEL_BUTTON_W, CONTROL_PANEL_SIDE_MARGIN, ORDER_TOOL_VIEWPORT_H,
        ORDER_TOOL_VIEWPORT_ID, ORDER_TOOL_VIEWPORT_W, SETTING_VIEWPORT_H, SETTING_VIEWPORT_ID, SETTING_VIEWPORT_W,
    },
    theme::_debug_check_rect,
};

pub struct MyApp {
    show_confirmation_dialog: Arc<AtomicBool>,
    allowed_to_close: Arc<AtomicBool>,
    always_on_top: bool,
    opened_viewports: Vec<egui::ViewportId>,
    // ----viewport 상태 변수----
    show_settings_viewport: Arc<AtomicBool>,
    show_account_viewport: Arc<AtomicBool>,
    show_order_tool_viewport: Arc<AtomicBool>,
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
            show_order_tool_viewport: Default::default(),
        }
    }
}

impl MyApp {
    fn render_control_panel(&mut self, ctx: &egui::Context) {
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

                // top bar
                ui.scope_builder(egui::UiBuilder::new().max_rect(top_rect), |ui| {
                    ui.with_layout(egui::Layout::left_to_right(egui::Align::Min), |ui| {
                        ui.spacing_mut().item_spacing.x = 1.0;

                        ui.add_space(CONTROL_PANEL_SIDE_MARGIN);
                        if ui.add_sized(top_left_btn_size, egui::Button::new("설정")).clicked() {
                            self.show_settings_viewport.store(true, Ordering::Relaxed);
                        }
                        if ui.add_sized(top_left_btn_size, egui::Button::new("계좌관리")).clicked() {
                            self.show_account_viewport.store(true, Ordering::Relaxed);
                        }
                        if ui.add_sized(top_left_btn_size, egui::Button::new("주문도구")).clicked() {
                            self.show_order_tool_viewport.store(true, Ordering::Relaxed);
                        }
                        // ui.add_sized(top_left_btn_size, egui::Button::new("주문체결"));
                        // ui.add_sized(top_left_btn_size, egui::Button::new("빠른호가"));
                        // ui.add_sized(top_left_btn_size, egui::Button::new("잔고손익"));

                        ui.add_space(ui.available_width());

                        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                            ui.add_space(CONTROL_PANEL_SIDE_MARGIN);

                            // always on top
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

                            // bring all to front
                            if ui.add_sized(top_right_btn_size, egui::Button::new("F")).clicked() {
                                for opened in &self.opened_viewports {
                                    ctx.send_viewport_cmd_to(*opened, egui::ViewportCommand::Visible(true));
                                    ctx.send_viewport_cmd_to(*opened, egui::ViewportCommand::Minimized(false));
                                    ctx.send_viewport_cmd_to(*opened, egui::ViewportCommand::Focus);
                                }
                            }
                        });
                    });
                });

                ui.painter().hline(
                    rect.x_range(),
                    split_y,
                    egui::Stroke::new(1.0, ui.visuals().widgets.noninteractive.bg_stroke.color),
                );

                // bottom
                ui.scope_builder(egui::UiBuilder::new().max_rect(bottom_rect), |ui| {
                    ui.with_layout(egui::Layout::left_to_right(egui::Align::Center), |ui| {
                        ui.add_space(CONTROL_PANEL_SIDE_MARGIN);
                        ui.label("[키움] Connection: OK | Token: OK | 12:34:56 (UTC+8)");
                    });
                });
            }); // end Central
    }

    fn render_settings_viewport(&mut self, ctx: &egui::Context) {
        let show_settings = Arc::clone(&self.show_settings_viewport); // 변경이 필요한 경우
        let viewport_id = egui::ViewportId::from_hash_of(SETTING_VIEWPORT_ID);
        let is_opened = self.show_settings_viewport.load(Ordering::Relaxed); // snapshot

        if !is_opened {
            self.opened_viewports.retain(|id| *id != viewport_id);
            return;
        }

        // --------------

        if !self.opened_viewports.contains(&viewport_id) {
            self.opened_viewports.push(viewport_id);
        }

        ctx.show_viewport_deferred(
            viewport_id,
            egui::ViewportBuilder::default()
                .with_title("설정")
                .with_inner_size([SETTING_VIEWPORT_W, SETTING_VIEWPORT_H]),
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

    fn render_account_viewport(&mut self, ctx: &egui::Context) {
        let show_settings = Arc::clone(&self.show_account_viewport); // 변경이 필요한 경우
        let viewport_id = egui::ViewportId::from_hash_of(ACCOUNT_VIEWPORT_ID);
        let is_opened = self.show_account_viewport.load(Ordering::Relaxed); // snapshot

        if !is_opened {
            self.opened_viewports.retain(|id| *id != viewport_id);
            return;
        }

        // --------------

        if !self.opened_viewports.contains(&viewport_id) {
            self.opened_viewports.push(viewport_id);
        }

        ctx.show_viewport_deferred(
            viewport_id,
            egui::ViewportBuilder::default()
                .with_title("계좌관리")
                .with_inner_size([ACCOUNT_VIEWPORT_W, ACCOUNT_VIEWPORT_H]),
            move |child_ctx, class| {
                // embedded는 허용하지 않지만 fallback으로 둔다
                if class == egui::ViewportClass::Embedded {
                    egui::Window::new("계좌관리").show(child_ctx, |ui| {
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

    fn render_order_tool_viewport(&mut self, ctx: &egui::Context) {
        let show_settings = Arc::clone(&self.show_order_tool_viewport); // 변경이 필요한 경우
        let viewport_id = egui::ViewportId::from_hash_of(ORDER_TOOL_VIEWPORT_ID);
        let is_opened = self.show_order_tool_viewport.load(Ordering::Relaxed); // snapshot

        if !is_opened {
            self.opened_viewports.retain(|id| *id != viewport_id);
            return;
        }

        // --------------

        if !self.opened_viewports.contains(&viewport_id) {
            self.opened_viewports.push(viewport_id);
        }

        ctx.show_viewport_deferred(
            viewport_id,
            egui::ViewportBuilder::default()
                .with_title("주문도구")
                .with_inner_size([ORDER_TOOL_VIEWPORT_W, ORDER_TOOL_VIEWPORT_H]),
            move |child_ctx, class| {
                // embedded는 허용하지 않지만 fallback으로 둔다
                if class == egui::ViewportClass::Embedded {
                    egui::Window::new("주문도구").show(child_ctx, |ui| {
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

    fn render_exit_confirm_viewport(&mut self, ctx: &egui::Context) {
        const EXIT_CONFIRM_VIEWPORT_ID: &str = "exit_confirm_viewport";
        let show_confirm = Arc::clone(&self.show_confirmation_dialog);
        let allowed_to_close = Arc::clone(&self.allowed_to_close);
        if !show_confirm.load(Ordering::Relaxed) {
            return;
        }
        ctx.show_viewport_deferred(
            egui::ViewportId::from_hash_of(EXIT_CONFIRM_VIEWPORT_ID),
            egui::ViewportBuilder::default()
                .with_title("종료 확인")
                .with_inner_size([200.0, 60.0])
                .with_resizable(false),
            move |child_ctx, class| {
                if class == egui::ViewportClass::Embedded {
                    egui::Window::new("종료 확인").show(child_ctx, |ui| {
                        ui.label("정말 종료하시겠습니까?");
                        ui.horizontal(|ui| {
                            if ui.button("종료").clicked() {
                                allowed_to_close.store(true, Ordering::Relaxed);
                                show_confirm.store(false, Ordering::Relaxed);
                                child_ctx.send_viewport_cmd_to(egui::ViewportId::ROOT, egui::ViewportCommand::Close);
                            }
                            if ui.button("취소").clicked() {
                                show_confirm.store(false, Ordering::Relaxed);
                            }
                        });
                    });
                } else {
                    egui::CentralPanel::default().show(child_ctx, |ui| {
                        ui.label("정말 종료하시겠습니까?");
                        ui.add_space(8.0);
                        ui.horizontal(|ui| {
                            if ui.button("종료").clicked() {
                                allowed_to_close.store(true, Ordering::Relaxed);
                                show_confirm.store(false, Ordering::Relaxed);
                                child_ctx.send_viewport_cmd_to(egui::ViewportId::ROOT, egui::ViewportCommand::Close);
                            }
                            if ui.button("취소").clicked() {
                                show_confirm.store(false, Ordering::Relaxed);
                            }
                        });
                    });
                }
                if child_ctx.input(|i| i.viewport().close_requested()) {
                    show_confirm.store(false, Ordering::Relaxed);
                }
            },
        );
    }
}
