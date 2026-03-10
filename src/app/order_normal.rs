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
const BUY_PRICE_TICK_OFFSETS: [i32; 5] = [-2, -1, 0, 1, 2];
const DEFAULT_BUY_PRICE_REF_LEVEL: i8 = 1;
const DEFAULT_BUY_PRICE_TICK_OFFSET: i32 = 0;

pub(super) fn render_order_normal_body(ui: &mut egui::Ui, ctx: &egui::Context, seq: u64) {
    let work_rect = ui.available_rect_before_wrap();
    let half_h = work_rect.height() * 0.5;

    let buy_rect = egui::Rect::from_min_max(work_rect.min, egui::pos2(work_rect.max.x, work_rect.min.y + half_h));
    let sell_rect = egui::Rect::from_min_max(egui::pos2(work_rect.min.x, work_rect.min.y + half_h), work_rect.max);

    ui.scope_builder(egui::UiBuilder::new().max_rect(buy_rect), |ui| {
        egui::Frame::new()
            .fill(egui::Color32::from_rgb(255, 240, 245))
            .stroke(egui::Stroke::new(1.0, egui::Color32::BLACK))
            .inner_margin(egui::Margin::same(8))
            .show(ui, |ui| {
                ui.set_min_size(ui.available_size());
                ui.vertical(|ui| {
                    ui.label(egui::RichText::new("매수 영역").strong());
                    ui.add_space(6.0);
                    render_once_investment_amount_input(ui, ctx, seq);
                    ui.add_space(6.0);
                    render_buy_price_inputs(ui, ctx, seq);
                });
            });
    });

    ui.scope_builder(egui::UiBuilder::new().max_rect(sell_rect), |ui| {
        egui::Frame::new()
            .fill(egui::Color32::from_rgb(235, 245, 255))
            .stroke(egui::Stroke::new(1.0, egui::Color32::BLACK))
            .inner_margin(egui::Margin::same(8))
            .show(ui, |ui| {
                ui.set_min_size(ui.available_size());
                ui.vertical(|ui| {
                    ui.label(egui::RichText::new("매도/청산/취소 영역").strong());
                    ui.add_space(6.0);
                    render_sell_rules_panel(ui, ctx, seq);
                });
            });
    });
}

fn render_sell_rules_panel(ui: &mut egui::Ui, ctx: &egui::Context, seq: u64) {
    render_take_profit_section(ui, ctx, seq);
    ui.add_space(6.0);
    render_breakeven_section(ui, ctx, seq);
    ui.add_space(6.0);
    render_stop_loss_section(ui, ctx, seq);
}

fn render_take_profit_section(ui: &mut egui::Ui, _ctx: &egui::Context, _seq: u64) {
    ui.group(|ui| {
        ui.label(egui::RichText::new("[자동] 익절 조건").strong());
    });
}

fn render_breakeven_section(ui: &mut egui::Ui, _ctx: &egui::Context, _seq: u64) {
    ui.group(|ui| {
        ui.label(egui::RichText::new("[자동] 본절 조건").strong());
    });
}

fn render_stop_loss_section(ui: &mut egui::Ui, _ctx: &egui::Context, _seq: u64) {
    ui.group(|ui| {
        ui.label(egui::RichText::new("[자동] 손절 조건").strong());
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

fn render_buy_price_inputs(ui: &mut egui::Ui, ctx: &egui::Context, seq: u64) {
    let ref_level_id = egui::Id::new(("order_tool_buy_price_ref_level", seq));
    let tick_offset_id = egui::Id::new(("order_tool_buy_price_tick_offset", seq));

    let mut ref_level = ctx
        .data_mut(|d| d.get_persisted::<i8>(ref_level_id))
        .unwrap_or(DEFAULT_BUY_PRICE_REF_LEVEL);
    let mut tick_offset = ctx
        .data_mut(|d| d.get_persisted::<i32>(tick_offset_id))
        .unwrap_or(DEFAULT_BUY_PRICE_TICK_OFFSET);

    if !BUY_PRICE_REF_LEVELS.iter().any(|(code, _)| *code == ref_level) {
        ref_level = DEFAULT_BUY_PRICE_REF_LEVEL;
    }
    if !BUY_PRICE_TICK_OFFSETS.contains(&tick_offset) {
        tick_offset = DEFAULT_BUY_PRICE_TICK_OFFSET;
    }

    ui.horizontal(|ui| {
        ui.label("매수가격");

        egui::ComboBox::from_id_salt(("order_tool_buy_price_ref_combo", seq))
            .selected_text(buy_price_ref_level_label(ref_level))
            .width(110.0)
            .show_ui(ui, |ui| {
                for (code, label) in BUY_PRICE_REF_LEVELS {
                    ui.selectable_value(&mut ref_level, code, label);
                }
            });

        egui::ComboBox::from_id_salt(("order_tool_buy_price_tick_combo", seq))
            .selected_text(format!("{tick_offset}틱"))
            .width(70.0)
            .show_ui(ui, |ui| {
                for offset in BUY_PRICE_TICK_OFFSETS {
                    ui.selectable_value(&mut tick_offset, offset, format!("{offset}틱"));
                }
            });
    });

    ctx.data_mut(|d| {
        d.insert_persisted(ref_level_id, ref_level);
        d.insert_persisted(tick_offset_id, tick_offset);
    });
}

fn buy_price_ref_level_label(code: i8) -> &'static str {
    BUY_PRICE_REF_LEVELS
        .iter()
        .find_map(|(level_code, label)| (*level_code == code).then_some(*label))
        .unwrap_or("매수1호가")
}
