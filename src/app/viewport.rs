use std::sync::{Arc, atomic::Ordering};

use eframe::egui;

use super::MyApp;
use crate::{
    constants::{
        ACCOUNT_VIEWPORT_H, ACCOUNT_VIEWPORT_ID, ACCOUNT_VIEWPORT_W, ORDER_TOOL_VIEWPORT_H, ORDER_TOOL_VIEWPORT_ID,
        ORDER_TOOL_VIEWPORT_W, SETTING_VIEWPORT_H, SETTING_VIEWPORT_ID, SETTING_VIEWPORT_W,
    },
    theme::_debug_check_rect,
};

impl MyApp {
    pub(super) fn render_settings_viewport(&mut self, ctx: &egui::Context) {
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

    pub(super) fn render_account_viewport(&mut self, ctx: &egui::Context) {
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

    pub(super) fn render_order_tool_viewport(&mut self, ctx: &egui::Context) {
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

    pub(super) fn render_exit_confirm_viewport(&mut self, ctx: &egui::Context) {
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
