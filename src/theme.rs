use eframe::egui::{self, FontData, FontDefinitions, FontFamily, Ui};

pub fn configure_fonts(ctx: &egui::Context) {
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

pub fn configure_sharp_style(ctx: &egui::Context) {
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

pub fn _debug_check_rect(ui: &Ui) {
    let panel_rect = ui.max_rect();
    let content_rect = ui.available_rect_before_wrap();

    //
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
