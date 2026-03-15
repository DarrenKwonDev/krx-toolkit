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
            let latest_raw_for_child = self.ws_latest_by_seq.get(&seq).cloned();
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

                            if let Some(raw_by_topic) = latest_raw_for_child.as_ref() {
                                if let Some(raw_0d) = find_latest_raw(raw_by_topic, ws_type::주식호가잔량, &selected_code)
                                {
                                    ui.add_space(2.0);
                                    ui.small(format!("WS[0D]: {}", ws_raw_summary(raw_0d)));
                                }
                                if let Some(raw_0b) = find_latest_raw(raw_by_topic, ws_type::주식체결, &selected_code) {
                                    ui.add_space(2.0);
                                    ui.small(format!("WS[0B]: {}", ws_raw_summary(raw_0b)));
                                }
                            }

                            let desired_topics = if selected_code.trim().is_empty() {
                                Vec::new()
                            } else {
                                vec![
                                    WsTopic {
                                        item: selected_code.clone(),
                                        ty: ws_type::주식호가잔량.to_owned(),
                                    },
                                    WsTopic {
                                        item: selected_code.clone(),
                                        ty: ws_type::주식체결.to_owned(),
                                    },
                                ]
                            };
                            ws_sync(&ws_cmd_tx_for_child, child_ctx, seq, desired_topics);
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
            self.ws_unsubscribe_all(seq);
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

fn ws_sync(
    ws_cmd_tx: &tokio::sync::mpsc::UnboundedSender<WsCommand>,
    child_ctx: &egui::Context,
    subscriber_id: u64,
    desired_topics: Vec<WsTopic>,
) {
    let prev_topics_id = egui::Id::new(("order_tool_prev_topics", subscriber_id));

    let prev_topic_keys = child_ctx
        .data_mut(|d| d.get_persisted::<Vec<String>>(prev_topics_id))
        .unwrap_or_default();

    let prev_topics = prev_topic_keys
        .iter()
        .filter_map(|key| parse_topic_key(key))
        .collect::<std::collections::HashSet<_>>();

    let next_topics = desired_topics
        .into_iter()
        .filter_map(|topic| {
            let ty = topic.ty.trim();
            let item = topic.item.trim();
            if ty.is_empty() || item.is_empty() {
                None
            } else {
                Some(WsTopic {
                    ty: ty.to_owned(),
                    item: item.to_owned(),
                })
            }
        })
        .collect::<std::collections::HashSet<_>>();

    let to_subscribe = next_topics.difference(&prev_topics).cloned().collect::<Vec<_>>();
    let to_unsubscribe = prev_topics.difference(&next_topics).cloned().collect::<Vec<_>>();

    ws_unsubscribe(ws_cmd_tx, subscriber_id, to_unsubscribe);
    ws_subscribe(ws_cmd_tx, subscriber_id, to_subscribe);

    let mut next_topic_keys = next_topics.into_iter().map(topic_to_key).collect::<Vec<_>>();
    next_topic_keys.sort();

    child_ctx.data_mut(|d| {
        d.insert_persisted(prev_topics_id, next_topic_keys);
    });
}

fn ws_subscribe(ws_cmd_tx: &tokio::sync::mpsc::UnboundedSender<WsCommand>, subscriber_id: u64, topics: Vec<WsTopic>) {
    if topics.is_empty() {
        return;
    }
    let _ = ws_cmd_tx.send(WsCommand::Subscribe { subscriber_id, topics });
}

fn ws_unsubscribe(ws_cmd_tx: &tokio::sync::mpsc::UnboundedSender<WsCommand>, subscriber_id: u64, topics: Vec<WsTopic>) {
    if topics.is_empty() {
        return;
    }
    let _ = ws_cmd_tx.send(WsCommand::Unsubscribe { subscriber_id, topics });
}

fn topic_to_key(topic: WsTopic) -> String {
    format!("{}|{}", topic.ty, topic.item)
}

fn parse_topic_key(key: &str) -> Option<WsTopic> {
    let (ty, item) = key.split_once('|')?;
    let ty = ty.trim();
    let item = item.trim();
    if ty.is_empty() || item.is_empty() {
        return None;
    }
    Some(WsTopic {
        ty: ty.to_owned(),
        item: item.to_owned(),
    })
}

fn ws_raw_summary(v: &serde_json::Value) -> String {
    let trnm = v.get("trnm").and_then(serde_json::Value::as_str).unwrap_or("-");
    let ty = v
        .get("type")
        .or_else(|| v.get("real_type"))
        .or_else(|| v.get("tr_type"))
        .or_else(|| v.get("ty"))
        .and_then(serde_json::Value::as_str)
        .unwrap_or("-");
    let item = v
        .get("item")
        .or_else(|| v.get("code"))
        .or_else(|| v.get("stk_cd"))
        .or_else(|| v.get("isu_cd"))
        .or_else(|| v.get("symbol"))
        .or_else(|| v.get("shrn_iscd"))
        .and_then(serde_json::Value::as_str)
        .unwrap_or("-");
    format!("trnm={trnm} type={ty} item={item}")
}

fn find_latest_raw<'a>(
    raw_by_topic: &'a std::collections::HashMap<WsTopic, serde_json::Value>,
    ty: &str,
    item: &str,
) -> Option<&'a serde_json::Value> {
    let item = item.trim();
    if item.is_empty() {
        return None;
    }

    let key = WsTopic {
        ty: ty.to_owned(),
        item: item.to_owned(),
    };
    raw_by_topic.get(&key)
}
