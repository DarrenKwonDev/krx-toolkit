use eframe::egui;

const DEFAULT_ONCE_INVESTMENT_AMOUNT: u64 = 100_000;
const BUY_PRICE_REF_LEVELS: [(i8, &str); 10] = [
    (-5, "매도5호가"),
    (-4, "매도4호가"),
    (-3, "매도3호가"),
    (-2, "매도2호가"),
    (-1, "매도1호가"),
    (1, "매수1호가"),
    (2, "매수2호가"),
    (3, "매수3호가"),
    (4, "매수4호가"),
    (5, "매수5호가"),
];
const DEFAULT_BUY_PRICE_REF_LEVEL: i8 = 1;
const SPLIT_BUY_ROW_COUNT: usize = 3;
const DEFAULT_SPLIT_BUY_WEIGHT_PCT: i32 = 0;
const DEFAULT_TAKE_PROFIT_PCT: i32 = 5;

pub(super) fn render_order_normal_body(
    ui: &mut egui::Ui,
    ctx: &egui::Context,
    seq: u64,
    latest_0d_raw: Option<&serde_json::Value>,
) {
    egui::Frame::new()
        .fill(egui::Color32::from_rgb(255, 240, 245))
        .stroke(egui::Stroke::new(1.0, egui::Color32::BLACK))
        .inner_margin(egui::Margin::same(8))
        .show(ui, |ui| {
            ui.set_min_width(ui.available_width());
            ui.vertical(|ui| {
                ui.label(egui::RichText::new("매수 영역").strong());
                ui.add_space(2.0);
                render_once_investment_amount_input(ui, ctx, seq);
                ui.add_space(2.0);
                render_buy_price_inputs(ui, ctx, seq, latest_0d_raw);
                ui.add_space(2.0);
                render_split_buy_rows(ui, ctx, seq);
                ui.add_space(2.0);
                ui.separator();
                let buy_all_btn = ui.add_sized([100.0, 24.0], egui::Button::new("일괄 매수 시행"));
                if buy_all_btn.clicked() {
                    // manual bulk buy entry point
                }
            });
        });

    ui.add_space(2.0);

    let sell_rect = ui.available_rect_before_wrap();
    ui.scope_builder(egui::UiBuilder::new().max_rect(sell_rect), |ui| {
        egui::Frame::new()
            .fill(egui::Color32::from_rgb(235, 245, 255))
            .stroke(egui::Stroke::new(1.0, egui::Color32::BLACK))
            .inner_margin(egui::Margin::same(8))
            .show(ui, |ui| {
                ui.set_min_size(ui.available_size());
                egui::ScrollArea::vertical().auto_shrink([false, false]).show(ui, |ui| {
                    ui.label(egui::RichText::new("매도/청산/취소 영역").strong());
                    ui.add_space(6.0);
                    render_sell_rules_panel(ui, ctx, seq);
                });
            });
    });
}

fn render_sell_rules_panel(ui: &mut egui::Ui, ctx: &egui::Context, seq: u64) {
    render_take_profit_section(ui, ctx, seq);
    ui.add_space(2.0);
    render_breakeven_section(ui, ctx, seq);
    ui.add_space(2.0);
    render_stop_loss_section(ui, ctx, seq);
    ui.add_space(2.0);
    ui.separator();
    ui.add_space(2.0);

    let liquidate_btn = ui.add_sized([190.0, 24.0], egui::Button::new("전량 시장가 매도 + 미체결취소"));
    if liquidate_btn.clicked() {
        // manual full liquidate + cancel all open orders entry point
    }
}

fn render_take_profit_section(ui: &mut egui::Ui, ctx: &egui::Context, seq: u64) {
    let trade_enabled_id = egui::Id::new(("order_normal_take_profit_trade_enabled", seq));
    let trade_pct_id = egui::Id::new(("order_normal_take_profit_trade_pct", seq));
    let trade_pct_draft_id = egui::Id::new(("order_normal_take_profit_trade_pct_draft", seq));
    let trade_amount_id = egui::Id::new(("order_normal_take_profit_trade_amount", seq));
    let trade_amount_draft_id = egui::Id::new(("order_normal_take_profit_trade_amount_draft", seq));

    let mut trade_enabled = ctx
        .data_mut(|d| d.get_persisted::<bool>(trade_enabled_id))
        .unwrap_or(false);
    let mut trade_pct = ctx
        .data_mut(|d| d.get_persisted::<i32>(trade_pct_id))
        .unwrap_or(DEFAULT_TAKE_PROFIT_PCT)
        .clamp(0, 100);
    let mut trade_pct_draft = ctx
        .data_mut(|d| d.get_persisted::<String>(trade_pct_draft_id))
        .unwrap_or_else(|| trade_pct.to_string());
    let mut trade_amount = ctx.data_mut(|d| d.get_persisted::<u64>(trade_amount_id)).unwrap_or(0);
    let mut trade_amount_draft = ctx
        .data_mut(|d| d.get_persisted::<String>(trade_amount_draft_id))
        .unwrap_or_else(|| trade_amount.to_string());

    egui::Frame::group(ui.style())
        .inner_margin(egui::Margin::same(8))
        .show(ui, |ui| {
            ui.set_min_width(ui.available_width());
            ui.label(egui::RichText::new("[자동] 익절 조건").strong());

            ui.horizontal(|ui| {
                ui.checkbox(&mut trade_enabled, "trade당 이익");
                ui.label("+");
                ui.add_enabled_ui(trade_enabled, |ui| {
                    render_percent_text_input(ui, &mut trade_pct, &mut trade_pct_draft);
                    ui.label("%");
                    ui.label("OR");
                    render_krw_text_input(ui, &mut trade_amount, &mut trade_amount_draft);
                    ui.label("원");
                });
            });
        });

    trade_pct = trade_pct.clamp(0, 100);

    ctx.data_mut(|d| {
        d.insert_persisted(trade_enabled_id, trade_enabled);
        d.insert_persisted(trade_pct_id, trade_pct);
        d.insert_persisted(trade_pct_draft_id, trade_pct_draft);
        d.insert_persisted(trade_amount_id, trade_amount);
        d.insert_persisted(trade_amount_draft_id, trade_amount_draft);
    });
}

fn render_breakeven_section(ui: &mut egui::Ui, ctx: &egui::Context, seq: u64) {
    let qty_opposite_enabled_id = egui::Id::new(("order_normal_breakeven_qty_opposite_enabled", seq));
    let mut qty_opposite_enabled = ctx
        .data_mut(|d| d.get_persisted::<bool>(qty_opposite_enabled_id))
        .unwrap_or(false);

    egui::Frame::group(ui.style())
        .inner_margin(egui::Margin::same(8))
        .show(ui, |ui| {
            ui.set_min_width(ui.available_width());
            ui.label(egui::RichText::new("[자동] 본절 조건").strong());
            ui.checkbox(&mut qty_opposite_enabled, "잔량 기준 반대 매매");
        });

    ctx.data_mut(|d| {
        d.insert_persisted(qty_opposite_enabled_id, qty_opposite_enabled);
    });
}

fn render_stop_loss_section(ui: &mut egui::Ui, ctx: &egui::Context, seq: u64) {
    let trade_enabled_id = egui::Id::new(("order_normal_stop_loss_trade_enabled", seq));
    let trade_pct_id = egui::Id::new(("order_normal_stop_loss_trade_pct", seq));
    let trade_pct_draft_id = egui::Id::new(("order_normal_stop_loss_trade_pct_draft", seq));
    let trade_amount_id = egui::Id::new(("order_normal_stop_loss_trade_amount", seq));
    let trade_amount_draft_id = egui::Id::new(("order_normal_stop_loss_trade_amount_draft", seq));

    let mut trade_enabled = ctx
        .data_mut(|d| d.get_persisted::<bool>(trade_enabled_id))
        .unwrap_or(false);
    let mut trade_pct = ctx
        .data_mut(|d| d.get_persisted::<i32>(trade_pct_id))
        .unwrap_or(DEFAULT_TAKE_PROFIT_PCT)
        .clamp(0, 100);
    let mut trade_pct_draft = ctx
        .data_mut(|d| d.get_persisted::<String>(trade_pct_draft_id))
        .unwrap_or_else(|| trade_pct.to_string());
    let mut trade_amount = ctx.data_mut(|d| d.get_persisted::<u64>(trade_amount_id)).unwrap_or(0);
    let mut trade_amount_draft = ctx
        .data_mut(|d| d.get_persisted::<String>(trade_amount_draft_id))
        .unwrap_or_else(|| trade_amount.to_string());

    egui::Frame::group(ui.style())
        .inner_margin(egui::Margin::same(8))
        .show(ui, |ui| {
            ui.set_min_width(ui.available_width());
            ui.label(egui::RichText::new("[자동] 손절 조건").strong());

            ui.horizontal(|ui| {
                ui.checkbox(&mut trade_enabled, "trade당 손실");
                ui.label("-");
                ui.add_enabled_ui(trade_enabled, |ui| {
                    render_percent_text_input(ui, &mut trade_pct, &mut trade_pct_draft);
                    ui.label("%");
                    ui.label("OR");
                    render_krw_text_input(ui, &mut trade_amount, &mut trade_amount_draft);
                    ui.label("원");
                });
            });
        });

    trade_pct = trade_pct.clamp(0, 100);

    ctx.data_mut(|d| {
        d.insert_persisted(trade_enabled_id, trade_enabled);
        d.insert_persisted(trade_pct_id, trade_pct);
        d.insert_persisted(trade_pct_draft_id, trade_pct_draft);
        d.insert_persisted(trade_amount_id, trade_amount);
        d.insert_persisted(trade_amount_draft_id, trade_amount_draft);
    });
}

fn render_once_investment_amount_input(ui: &mut egui::Ui, ctx: &egui::Context, seq: u64) {
    let value_id = egui::Id::new(("order_tool_once_investment_amount_value", seq));
    let draft_id = egui::Id::new(("order_tool_once_investment_amount_draft", seq));

    let mut value = ctx
        .data_mut(|d| d.get_persisted::<u64>(value_id))
        .unwrap_or(DEFAULT_ONCE_INVESTMENT_AMOUNT);
    let mut draft = ctx
        .data_mut(|d| d.get_persisted::<String>(draft_id))
        .unwrap_or_else(|| value.to_string());

    ui.horizontal(|ui| {
        ui.label("1회 투자 금액");
        let response = ui.add_sized([120.0, 20.0], egui::TextEdit::singleline(&mut draft));
        ui.label("원");

        let commit_by_enter = response.has_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter));
        let commit_by_focus_out = response.lost_focus() && !response.has_focus();

        if commit_by_enter || commit_by_focus_out {
            if let Some(parsed) = parse_amount(&draft) {
                value = parsed;
                draft = value.to_string();
            } else {
                draft = value.to_string();
            }
        }
    });

    ctx.data_mut(|d| {
        d.insert_persisted(value_id, value);
        d.insert_persisted(draft_id, draft);
    });
}

fn parse_amount(raw: &str) -> Option<u64> {
    let normalized = raw.trim().replace([',', '_', ' '], "");
    let parsed = normalized.parse::<u64>().ok()?;
    (parsed > 0).then_some(parsed)
}

fn render_buy_price_inputs(ui: &mut egui::Ui, ctx: &egui::Context, seq: u64, latest_0d_raw: Option<&serde_json::Value>) {
    let ref_level_id = egui::Id::new(("order_tool_buy_price_ref_level", seq));

    let mut ref_level = ctx
        .data_mut(|d| d.get_persisted::<i8>(ref_level_id))
        .unwrap_or(DEFAULT_BUY_PRICE_REF_LEVEL);

    if !BUY_PRICE_REF_LEVELS.iter().any(|(code, _)| *code == ref_level) {
        ref_level = DEFAULT_BUY_PRICE_REF_LEVEL;
    }

    ui.horizontal(|ui| {
        ui.label("공통 매수가격");

        egui::ComboBox::from_id_salt(("order_tool_buy_price_ref_combo", seq))
            .selected_text(buy_price_ref_level_label(ref_level))
            .width(110.0)
            .show_ui(ui, |ui| {
                for (code, label) in BUY_PRICE_REF_LEVELS {
                    ui.selectable_value(&mut ref_level, code, label);
                }
            });

        let ref_price = price_from_0d_by_ref_level(latest_0d_raw, ref_level)
            .map(format_price_text)
            .unwrap_or_else(|| "-".to_owned());
        ui.label(format!("= {ref_price}"));
    });

    ctx.data_mut(|d| {
        d.insert_persisted(ref_level_id, ref_level);
    });
}

fn price_from_0d_by_ref_level(latest_0d_raw: Option<&serde_json::Value>, ref_level: i8) -> Option<&str> {
    let values = latest_0d_raw?.get("values")?.as_object()?;
    let key = ref_level_to_0d_key(ref_level)?;
    values.get(key).and_then(serde_json::Value::as_str)
}

fn ref_level_to_0d_key(ref_level: i8) -> Option<&'static str> {
    match ref_level {
        -5 => Some("45"),
        -4 => Some("44"),
        -3 => Some("43"),
        -2 => Some("42"),
        -1 => Some("41"),
        1 => Some("51"),
        2 => Some("52"),
        3 => Some("53"),
        4 => Some("54"),
        5 => Some("55"),
        _ => None,
    }
}

fn format_price_text(raw: &str) -> String {
    let normalized = raw.trim().replace(',', "");
    if normalized.is_empty() {
        return "-".to_owned();
    }

    let Ok(parsed) = normalized.parse::<i64>() else {
        return normalized;
    };

    format_i64_with_commas(parsed.abs())
}

fn format_i64_with_commas(value: i64) -> String {
    let s = value.to_string();
    let mut out = String::with_capacity(s.len() + (s.len().saturating_sub(1) / 3));
    for (idx, ch) in s.chars().rev().enumerate() {
        if idx > 0 && idx % 3 == 0 {
            out.push(',');
        }
        out.push(ch);
    }
    out.chars().rev().collect()
}

fn render_split_buy_rows(ui: &mut egui::Ui, ctx: &egui::Context, seq: u64) {
    for row_index in 0..SPLIT_BUY_ROW_COUNT {
        render_split_buy_row(ui, ctx, seq, row_index);
        if row_index + 1 < SPLIT_BUY_ROW_COUNT {
            ui.add_space(4.0);
        }
    }
}

fn render_split_buy_row(ui: &mut egui::Ui, ctx: &egui::Context, seq: u64, row_index: usize) {
    let weight_pct_id = egui::Id::new(("order_tool_split_buy_weight_pct", seq, row_index));
    let weight_pct_draft_id = egui::Id::new(("order_tool_split_buy_weight_pct_draft", seq, row_index));

    let mut weight_pct = ctx
        .data_mut(|d| d.get_persisted::<i32>(weight_pct_id))
        .unwrap_or(DEFAULT_SPLIT_BUY_WEIGHT_PCT)
        .clamp(0, 100);
    let mut weight_pct_draft = ctx
        .data_mut(|d| d.get_persisted::<String>(weight_pct_draft_id))
        .unwrap_or_else(|| weight_pct.to_string());

    ui.horizontal(|ui| {
        ui.label(format!("분할매수{}", row_index + 1));
        render_percent_text_input(ui, &mut weight_pct, &mut weight_pct_draft);
        ui.label("%");

        let buy_btn = ui.add_sized([80.0, 24.0], egui::Button::new("매수 시행"));
        if buy_btn.clicked() {
            // manual split buy entry point
        }
    });

    weight_pct = weight_pct.clamp(0, 100);

    ctx.data_mut(|d| {
        d.insert_persisted(weight_pct_id, weight_pct);
        d.insert_persisted(weight_pct_draft_id, weight_pct_draft);
    });
}

fn buy_price_ref_level_label(code: i8) -> &'static str {
    BUY_PRICE_REF_LEVELS
        .iter()
        .find_map(|(level_code, label)| (*level_code == code).then_some(*label))
        .unwrap_or("매수1호가")
}

fn render_percent_text_input(ui: &mut egui::Ui, value: &mut i32, draft: &mut String) {
    let response = ui.add_sized([48.0, 20.0], egui::TextEdit::singleline(draft));

    let commit_by_enter = response.has_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter));
    let commit_by_focus_out = response.lost_focus() && !response.has_focus();

    if commit_by_enter || commit_by_focus_out {
        if let Some(parsed) = parse_percent(draft) {
            *value = parsed;
            *draft = value.to_string();
        } else {
            *draft = value.to_string();
        }
    }
}

fn parse_percent(raw: &str) -> Option<i32> {
    let normalized = raw.trim().replace(['%', ',', '_', ' '], "");
    let parsed = normalized.parse::<i32>().ok()?;
    Some(parsed.clamp(0, 100))
}

fn render_krw_text_input(ui: &mut egui::Ui, value: &mut u64, draft: &mut String) {
    let response = ui.add_sized([72.0, 20.0], egui::TextEdit::singleline(draft));

    let commit_by_enter = response.has_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter));
    let commit_by_focus_out = response.lost_focus() && !response.has_focus();

    if commit_by_enter || commit_by_focus_out {
        if let Some(parsed) = parse_krw_amount(draft) {
            *value = parsed;
            *draft = value.to_string();
        } else {
            *draft = value.to_string();
        }
    }
}

fn parse_krw_amount(raw: &str) -> Option<u64> {
    let normalized = raw.trim().replace([',', '_', ' '], "");
    normalized.parse::<u64>().ok()
}
