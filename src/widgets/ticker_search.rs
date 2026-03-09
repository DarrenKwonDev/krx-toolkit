use std::collections::HashSet;

use eframe::egui;

use crate::app::MasterData;

#[derive(Debug, Clone)]
pub struct TickerSelection {
    pub code: String,
    pub name: String,
    pub market: String,
}

#[derive(Debug, Clone, Default)]
pub struct TickerSearchOutput {
    pub selected: Option<TickerSelection>,
}

pub fn render_ticker_search(
    ui: &mut egui::Ui,
    ctx: &egui::Context,
    master: &MasterData,
    seq: u64,
    popup_width: f32,
    max_results: usize,
) -> TickerSearchOutput {
    let query_id = egui::Id::new(("order_tool_query", seq));
    let popup_id = egui::Id::new(("order_tool_picker_popup", seq));
    let selected_code_id = egui::Id::new(("order_tool_selected_code", seq));
    let selected_name_id = egui::Id::new(("order_tool_selected_name", seq));
    let selected_market_id = egui::Id::new(("order_tool_selected_market", seq));

    let mut query = ctx
        .data_mut(|d| d.get_persisted::<String>(query_id))
        .unwrap_or_default();
    let mut selected_code = ctx
        .data_mut(|d| d.get_persisted::<String>(selected_code_id))
        .unwrap_or_default();
    let mut selected_name = ctx
        .data_mut(|d| d.get_persisted::<String>(selected_name_id))
        .unwrap_or_default();
    let mut selected_market = ctx
        .data_mut(|d| d.get_persisted::<String>(selected_market_id))
        .unwrap_or_default();

    ui.horizontal(|ui| {
        let query_resp = ui.add_sized(
            [100.0, 18.0],
            egui::TextEdit::singleline(&mut query).hint_text("종목코드/종목명"),
        );

        if query_resp.changed() {
            if query.trim().is_empty() {
                egui::Popup::close_id(ctx, popup_id);
            } else {
                egui::Popup::open_id(ctx, popup_id);
            }
        }

        if ui.add_sized([18.0, 18.0], egui::Button::new("종")).clicked() {
            egui::Popup::toggle_id(ctx, popup_id);
        }

        egui::Popup::from_response(&query_resp)
            .id(popup_id)
            .open_memory(None)
            .align(egui::RectAlign::BOTTOM_START)
            .width(popup_width)
            .close_behavior(egui::PopupCloseBehavior::CloseOnClickOutside)
            .show(|ui| {
                ui.set_min_width(popup_width);
                egui::Frame::new()
                    .fill(egui::Color32::WHITE)
                    .stroke(egui::Stroke::new(1.0, egui::Color32::BLACK))
                    .inner_margin(egui::Margin::same(4))
                    .show(ui, |ui| {
                        ui.set_min_width((popup_width - 8.0).max(0.0));

                        let matches = collect_ticker_matches(master, &query, max_results);
                        let mut picked: Option<TickerSelection> = None;
                        let popup_inner_w = (popup_width - 8.0).max(0.0);

                        egui::ScrollArea::vertical()
                            .auto_shrink([false, false])
                            .max_height(130.0)
                            .show(ui, |ui| {
                                ui.set_min_width(popup_inner_w);
                                if matches.is_empty() {
                                    ui.label("매칭되는 종목이 없습니다.");
                                } else {
                                    for item in matches {
                                        let selected = selected_code == item.code;
                                        let label = format!("{} | {} [{}]", item.code, item.name, item.market);
                                        if ui.selectable_label(selected, label).clicked() {
                                            picked = Some(item);
                                        }
                                    }
                                }
                            });

                        if let Some(item) = picked {
                            query = item.code.clone();
                            selected_code = item.code;
                            selected_name = item.name;
                            selected_market = item.market;
                            egui::Popup::close_id(ctx, popup_id);
                        }
                    });
            });

        if ui.input(|i| i.key_pressed(egui::Key::Escape)) {
            egui::Popup::close_id(ctx, popup_id);
        }
        if query.trim().is_empty() {
            egui::Popup::close_id(ctx, popup_id);
        }
    });

    ctx.data_mut(|d| {
        d.insert_persisted(query_id, query.clone());
        d.insert_persisted(selected_code_id, selected_code.clone());
        d.insert_persisted(selected_name_id, selected_name.clone());
        d.insert_persisted(selected_market_id, selected_market.clone());
    });

    TickerSearchOutput {
        selected: (!selected_code.is_empty()).then(|| TickerSelection {
            code: selected_code,
            name: selected_name,
            market: selected_market,
        }),
    }
}

fn collect_ticker_matches(master: &MasterData, query: &str, limit: usize) -> Vec<TickerSelection> {
    if limit == 0 {
        return Vec::new();
    }

    let q = query.trim();
    if q.is_empty() {
        return Vec::new();
    }

    let q_lower = q.to_lowercase();
    let mut out = Vec::with_capacity(limit.min(64));
    let mut seen_codes: HashSet<String> = HashSet::new();

    for page in &master.kospi_pages {
        if collect_from_list_page(page, "KOSPI", q, &q_lower, &mut seen_codes, &mut out, limit) {
            return out;
        }
    }
    for page in &master.kosdaq_pages {
        if collect_from_list_page(page, "KOSDAQ", q, &q_lower, &mut seen_codes, &mut out, limit) {
            return out;
        }
    }

    out
}

fn collect_from_list_page(
    page: &serde_json::Value,
    market: &str,
    query: &str,
    query_lower: &str,
    seen_codes: &mut HashSet<String>,
    out: &mut Vec<TickerSelection>,
    limit: usize,
) -> bool {
    if out.len() >= limit {
        return true;
    }

    let Some(rows) = page.get("list").and_then(|v| v.as_array()) else {
        return false;
    };

    for row in rows {
        let Some(map) = row.as_object() else {
            continue;
        };

        let Some(code_raw) = map.get("code").and_then(|v| v.as_str()) else {
            continue;
        };
        let Some(name_raw) = map.get("name").and_then(|v| v.as_str()) else {
            continue;
        };

        let code = code_raw.trim();
        let name = name_raw.trim();
        if code.is_empty() || name.is_empty() {
            continue;
        }

        let name_lower = name.to_lowercase();
        let matched = code.starts_with(query) || name_lower.contains(query_lower);
        if !matched {
            continue;
        }

        if seen_codes.insert(code.to_owned()) {
            let market_label = map
                .get("marketName")
                .and_then(|v| v.as_str())
                .map(str::trim)
                .filter(|s| !s.is_empty())
                .unwrap_or(market)
                .to_owned();

            out.push(TickerSelection {
                code: code.to_owned(),
                name: name.to_owned(),
                market: market_label,
            });
            if out.len() >= limit {
                return true;
            }
        }
    }

    false
}
