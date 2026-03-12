use std::sync::{
    Arc,
    atomic::{AtomicBool, Ordering},
};

use eframe::egui;

use super::MyApp;
use super::order_normal::render_order_normal_body;
use crate::{
    api::kiwoom::ws::ws_type,
    constants::{
        ACCOUNT_VIEWPORT_H, ACCOUNT_VIEWPORT_ID, ACCOUNT_VIEWPORT_W, EMERGENCY_ORDER_VIEWPORT_H,
        EMERGENCY_ORDER_VIEWPORT_ID, EMERGENCY_ORDER_VIEWPORT_W, ORDER_TOOL_SEARCH_POPUP_W, ORDER_TOOL_VIEWPORT_H,
        ORDER_TOOL_VIEWPORT_ID, ORDER_TOOL_VIEWPORT_W, SETTING_VIEWPORT_H, SETTING_VIEWPORT_ID, SETTING_VIEWPORT_W,
    },
    tasks::ws_task::{WsCommand, WsTopic},
    theme::_debug_check_rect,
    widgets::ticker_search::render_ticker_search,
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
                .with_title("계좌상태")
                .with_inner_size([ACCOUNT_VIEWPORT_W, ACCOUNT_VIEWPORT_H]),
            move |child_ctx, class| {
                // embedded는 허용하지 않지만 fallback으로 둔다
                if class == egui::ViewportClass::Embedded {
                    egui::Window::new("계좌상태").show(child_ctx, |ui| {
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

    pub(super) fn open_new_order_tool_viewport(&mut self) {
        self.next_order_tool_seq += 1;
        let seq = self.next_order_tool_seq;
        let id = egui::ViewportId::from_hash_of(format!("{}_{}", ORDER_TOOL_VIEWPORT_ID, self.next_order_tool_seq));
        let open = Arc::new(AtomicBool::new(true));
        self.order_tool_viewports.push((id, open, seq));

        // 열린 창 리스트에 추가
        if !self.opened_viewports.contains(&id) {
            self.opened_viewports.push(id);
        }
    }

    pub(super) fn render_emergency_order_viewport(&mut self, ctx: &egui::Context) {
        let show_emergency = Arc::clone(&self.show_emergency_order_viewport);
        let viewport_id = egui::ViewportId::from_hash_of(EMERGENCY_ORDER_VIEWPORT_ID);
        let is_opened = self.show_emergency_order_viewport.load(Ordering::Relaxed);

        if !is_opened {
            self.opened_viewports.retain(|id| *id != viewport_id);
            return;
        }

        if !self.opened_viewports.contains(&viewport_id) {
            self.opened_viewports.push(viewport_id);
        }

        ctx.show_viewport_deferred(
            viewport_id,
            egui::ViewportBuilder::default()
                .with_title("주문[긴급]")
                .with_inner_size([EMERGENCY_ORDER_VIEWPORT_W, EMERGENCY_ORDER_VIEWPORT_H]),
            move |child_ctx, class| {
                if class == egui::ViewportClass::Embedded {
                    egui::Window::new("주문[긴급]").show(child_ctx, |ui| {
                        ui.label("emergency order placeholder");
                    });
                } else {
                    egui::CentralPanel::default().show(child_ctx, |ui| {
                        _debug_check_rect(ui);
                    });
                }

                if child_ctx.input(|i| i.viewport().close_requested()) {
                    show_emergency.store(false, Ordering::Relaxed);
                }
            },
        );
    }

    pub(super) fn render_order_tool_viewport(&mut self, ctx: &egui::Context) {
        let viewports = self.order_tool_viewports.clone();
        let master = Arc::clone(&self.master);

        for (viewport_id, is_open, seq) in viewports {
            if !is_open.load(Ordering::Relaxed) {
                continue;
            }

            if !self.opened_viewports.contains(&viewport_id) {
                self.opened_viewports.push(viewport_id);
            }

            let open_for_child = Arc::clone(&is_open);
            let master_for_child = Arc::clone(&master);
            let ws_cmd_tx_for_child = self.ws_cmd_tx.clone();
            ctx.show_viewport_deferred(
                viewport_id,
                egui::ViewportBuilder::default()
                    .with_title(format!("{}#{}", "주문도구", seq))
                    .with_inner_size([ORDER_TOOL_VIEWPORT_W, ORDER_TOOL_VIEWPORT_H]),
                move |child_ctx, class| {
                    if class == egui::ViewportClass::Embedded {
                        egui::Window::new("주문도구").show(child_ctx, |ui| {
                            ui.label("settings placeholder");
                        });
                    } else {
                        // ticker picker
                        egui::TopBottomPanel::top(egui::Id::new(("order_tool_top", seq))).show(child_ctx, |ui| {
                            let output = render_ticker_search(
                                ui,
                                child_ctx,
                                master_for_child.as_ref(),
                                seq,
                                ORDER_TOOL_SEARCH_POPUP_W,
                                30,
                            );

                            if let Some(ref selected) = output.selected {
                                ui.add_space(4.0);
                                ui.label(format!("선택: {} | {}", selected.code, selected.name));
                            }

                            let selected_code_id = egui::Id::new(("order_tool_selected_code", seq));
                            let selected_code = child_ctx
                                .data_mut(|d| d.get_persisted::<String>(selected_code_id))
                                .unwrap_or_default();
                            let prev_subscribed_id = egui::Id::new(("order_tool_prev_subscribed_code", seq));
                            let prev_subscribed = child_ctx
                                .data_mut(|d| d.get_persisted::<String>(prev_subscribed_id))
                                .unwrap_or_default();

                            if selected_code != prev_subscribed {
                                if !prev_subscribed.is_empty() {
                                    let _ = ws_cmd_tx_for_child.send(WsCommand::Unsubscribe {
                                        subscriber_id: seq,
                                        topics: vec![WsTopic {
                                            item: prev_subscribed.clone(),
                                            ty: ws_type::주식호가잔량.to_owned(),
                                        }],
                                    });
                                }

                                if !selected_code.is_empty() {
                                    let _ = ws_cmd_tx_for_child.send(WsCommand::Subscribe {
                                        subscriber_id: seq,
                                        topics: vec![WsTopic {
                                            item: selected_code.clone(),
                                            ty: ws_type::주식호가잔량.to_owned(),
                                        }],
                                    });
                                }

                                child_ctx.data_mut(|d| {
                                    d.insert_persisted(prev_subscribed_id, selected_code);
                                });
                            }
                        });

                        // actual order tools body
                        egui::CentralPanel::default().show(child_ctx, |ui| {
                            render_order_normal_body(ui, child_ctx, seq);
                        });
                    }
                    if child_ctx.input(|i| i.viewport().close_requested()) {
                        open_for_child.store(false, Ordering::Relaxed);
                    }
                },
            );
        }

        // 닫혀야 할 viewport를 삭제한다
        let closed = self
            .order_tool_viewports
            .iter()
            .filter_map(|(id, is_open, seq)| (!is_open.load(Ordering::Relaxed)).then_some((*id, *seq)))
            .collect::<Vec<_>>();

        self.order_tool_viewports
            .retain(|(_, is_open, _)| is_open.load(Ordering::Relaxed));

        for (id, seq) in closed {
            self.unsubscribe_all_for_viewport(seq);
            self.opened_viewports.retain(|opened| *opened != id);
        }
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
                                child_ctx.send_viewport_cmd(egui::ViewportCommand::Close);
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
                                child_ctx.send_viewport_cmd(egui::ViewportCommand::Close);
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
