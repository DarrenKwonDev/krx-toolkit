use std::sync::atomic::Ordering;

use eframe::egui;

use super::MyApp;
use crate::constants::{
    CONTROL_PANEL_BOTTOM_MARGIN, CONTROL_PANEL_BUTTON_H, CONTROL_PANEL_BUTTON_W, CONTROL_PANEL_SIDE_MARGIN,
};

impl MyApp {
    pub(super) fn render_control_panel(&mut self, ctx: &egui::Context) {
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
                            // self.show_order_tool_viewport.store(true, Ordering::Relaxed);
                            self.open_new_order_tool_viewport();
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
}
