use super::common::{mi, mi_text, stack};
use super::{CONTENT_H, Rect, Sense, Target, Ui, WordApp, big, group, json, menu_button, small, vec2};

/// The Design tab's style set gallery in a box of its own, sized from its thumbnails and clipped
/// to it, with a gap on each side: the Colors and Fonts column can't be drawn over, or cut by,
/// the thumbnails (#638). In a right-to-left interface, the thumbnails lay out from the right
/// (#5750). Returns the box and the space the thumbnails used.
pub(super) fn style_set_box(app: &mut WordApp, ui: &mut Ui) -> (Rect, Rect) {
    ui.add_space(4.0);
    let (gallery, _) = ui.allocate_exact_size(vec2(crate::previews::STYLE_SET_GALLERY_W, CONTENT_H), Sense::hover());
    let mut gui = ui.new_child(egui::UiBuilder::new().max_rect(gallery).layout(crate::rtl::row_layout(ui, egui::Align::Min)));
    gui.set_clip_rect(gallery.intersect(ui.clip_rect()));
    gui.spacing_mut().item_spacing = vec2(1.0, 1.0);
    crate::previews::style_set_gallery(app, &mut gui);
    ui.add_space(8.0);
    (gallery, gui.min_rect())
}

/// The Themes or Colors menu: each theme with its colours, ours then the user's, and the item
/// that makes a new one.
fn theme_menu(ui: &mut Ui, app: &mut WordApp, id: &str, create: &str) {
    let all = wordcraft_engine::cmd::design::themes(&app.session);
    let current = app.session.doc.settings.theme_name.clone();
    for (heading, custom) in [("Built-In", false), ("Custom", true)] {
        let list: Vec<_> = all.iter().filter(|t| t.custom == custom).collect();
        if list.is_empty() {
            continue;
        }
        ui.label(egui::RichText::new(tl!(heading)).font(crate::theme::semibold(11.5)).color(crate::theme::Tokens::get(ui.ctx()).text_dim));
        for t in list {
            theme_item(ui, app, t, id, t.name.eq_ignore_ascii_case(&current));
        }
    }
    ui.separator();
    mi(ui, app, create, "ui.dialog", json!({"name": "createTheme"}));
}

/// A theme menu entry: its six accent colours, then its name; the document's theme is outlined.
fn theme_item(ui: &mut Ui, app: &mut WordApp, t: &wordcraft_engine::cmd::design::Theme, id: &str, current: bool) {
    use egui::{Stroke, pos2};
    let name = if t.custom { t.name.clone() } else { tl!(t.name.as_str()).to_string() };
    let tok = crate::theme::Tokens::get(ui.ctx());
    let galley = ui.painter().layout_no_wrap(name.clone(), egui::TextStyle::Button.resolve(ui.style()), tok.text);
    let (size, gap) = (12.0, 3.0);
    let swatches = 6.0 * (size + gap);
    let (rect, resp) = ui.allocate_exact_size(vec2((swatches + galley.size().x + 24.0).max(220.0), 24.0), Sense::click());
    resp.widget_info(|| egui::WidgetInfo::labeled(egui::Role::Button, true, &name));
    let painter = ui.painter();
    if resp.hovered() {
        painter.rect_filled(rect, 3.0, tok.hover);
    }
    if current {
        painter.rect_stroke(rect.shrink(1.0), 3.0, Stroke::new(1.0, tok.accent), egui::StrokeKind::Inside);
    }
    let mut x = rect.left() + 8.0;
    for c in t.accents {
        let r = Rect::from_min_size(pos2(x, rect.center().y - size / 2.0), vec2(size, size));
        painter.rect_filled(r, 2.0, crate::theme::c32(c));
        painter.rect_stroke(r, 2.0, Stroke::new(0.5, tok.border_strong), egui::StrokeKind::Inside);
        x += size + gap;
    }
    painter.galley(pos2(x + 6.0, rect.center().y - galley.size().y / 2.0), galley, tok.text);
    if resp.clicked() {
        crate::widgets::run_from_ribbon(app, id, json!({"name": t.name}));
        ui.close();
    }
}

pub(super) fn design(app: &mut WordApp, ui: &mut Ui) {
    group(ui, "Themes", None, app, |ui, app| {
        menu_button(ui, app, crate::widgets::MenuButtonOptions::new("themes", Some("Themes"), "Themes", true, "design.themes"), |ui, app| {
            theme_menu(ui, app, "design.theme", "Create New Theme…");
        });
    });
    group(ui, "Document Formatting", None, app, |ui, app| {
        style_set_box(app, ui);
        stack(ui, |ui| {
            menu_button(
                ui,
                app,
                crate::widgets::MenuButtonOptions::new("colors", Some("Colors"), "Theme Colors", false, "design.themeColors"),
                |ui, app| {
                    theme_menu(ui, app, "design.themeColors", "Customize Colors…");
                },
            );
            menu_button(
                ui,
                app,
                crate::widgets::MenuButtonOptions::new("fonts", Some("Fonts"), "Theme Fonts", false, "design.themeFonts"),
                |ui, app| {
                    for t in wordcraft_engine::cmd::design::themes(&app.session) {
                        let (name, h, b) = (if t.custom { t.name.clone() } else { tl!(t.name.as_str()).to_string() }, &t.heading, &t.body);
                        mi_text(ui, app, &format!("{name}: {h} / {b}"), "design.themeFonts", json!({"heading": h, "body": b}));
                    }
                },
            );
        });
        stack(ui, |ui| {
            menu_button(
                ui,
                app,
                crate::widgets::MenuButtonOptions::new(
                    "paraSpacing",
                    Some("Paragraph Spacing"),
                    "Paragraph Spacing",
                    false,
                    "design.paragraphSpacing",
                ),
                |ui, app| {
                    for (l, v) in [
                        ("Default", "default"),
                        ("No Paragraph Space", "none"),
                        ("Compact", "compact"),
                        ("Tight", "tight"),
                        ("Open", "open"),
                        ("Relaxed", "relaxed"),
                        ("Double", "double"),
                    ] {
                        // "Open" here is roomy spacing, not the file verb: only a `spacing` row
                        // translates it, else it stays English.
                        if l == "Open" {
                            mi_text(ui, app, crate::i18n::get("spacing", l).unwrap_or(l), "design.paragraphSpacing", json!({"value": v}));
                            continue;
                        }
                        mi(ui, app, l, "design.paragraphSpacing", json!({"value": v}));
                    }
                },
            );
            small(ui, app, "effectsDesign", Some("Effects"), "Effects", crate::widgets::ButtonAction::new("design.effects", json!({}), false));
            small(
                ui,
                app,
                "setDefault",
                Some("Set as Default"),
                "Set as Default",
                crate::widgets::ButtonAction::new("design.setDefault", json!({}), false),
            );
        });
    });
    group(ui, "Page Background", None, app, |ui, app| {
        menu_button(
            ui,
            app,
            crate::widgets::MenuButtonOptions::new("watermark", Some("Watermark"), "Watermark", true, "design.watermark"),
            |ui, app| {
                for w in ["DRAFT", "CONFIDENTIAL", "DO NOT COPY", "SAMPLE", "ASAP", "URGENT"] {
                    mi(ui, app, w, "design.watermark", json!({"text": w}));
                }
                ui.separator();
                mi(ui, app, "Custom Watermark…", "ui.dialog", json!({"name": "watermark"}));
                mi(ui, app, "Remove Watermark", "design.watermark", json!({"remove": true}));
            },
        );
        menu_button(
            ui,
            app,
            crate::widgets::MenuButtonOptions::new("pageColor", Some("Page\nColor"), "Page Color", true, "design.pageColor"),
            |ui, app| {
                mi(ui, app, "No Color", "design.pageColor", json!({"color": null}));
                let theme = app.session.doc.settings.theme_colors.clone();
                crate::colors::menu(ui, app, &theme, Target::cmd("design.pageColor"));
            },
        );
        big(ui, app, "pageBorders", "Page\nBorders", "design.pageBorders", json!({}), false);
    });
}
