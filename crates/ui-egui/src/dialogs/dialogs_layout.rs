//! Layout, Review and Mailings dialogs (#407): Line Numbers (Layout › Line Numbers › Line
//! Numbering Options…), Hyphenation (Hyphenation Options… and Manual…), Language (Review ›
//! Language), Envelopes and Labels (Mailings › Create), Formula (Table Layout › Data, #1201)
//! Sort (Table Layout › Data, #1200), Split Cells (Table Layout › Merge), Convert Text to Table
//! (Insert › Table, #2592), Size (Picture Format › Size, #2573) and Format Page Numbers (Insert ›
//! Header & Footer › Page Number ▸ Format Page Numbers…, #1113). Each ends by running a
//! command (`layout.lineNumbers`, `layout.hyphenation`, `layout.manualHyphenation` with
//! `caret.set` and `text.optionalHyphen`, `review.language`, `mailings.envelopes`,
//! `mailings.labels`, `table.formula`, `table.sort`, `table.split`, `table.fromText`,
//! `picture.size`, `layout.pageNumberFormat`), so scripts and agents get the same result without
//! the dialog; Create New Theme (Design › Themes and Colors) runs `design.createTheme`. Shown through [`crate::dialogs::Dialog::Layout`].

use egui::{Ui, vec2};
use serde::Serialize;
use serde_json::{Value, json};

use wordcraft_doc::section::LineNumberRestart;
use wordcraft_engine::cmd::mailings::{ENVELOPE_SIZES, LABEL_PRODUCTS};
use wordcraft_engine::cmd::objects::fit_size;

use crate::WordApp;
use crate::options::MeasureExt;
use crate::theme::semibold;

mod page;
mod print;
mod table;
mod text;

pub use page::{PageNumbersForm, SizeForm};
use page::{page_numbers_ui, size_ui};
#[cfg(test)]
pub(crate) use print::envelope_label;
#[cfg(test)]
use print::product_label;
pub use print::{EnvelopeForm, LabelsForm};
use print::{envelopes_ui, labels_ui};
pub use table::{FORMULA_FUNCTIONS, FormulaForm, SortForm, SplitCellsForm, TableToTextForm, TextToTableForm};
use table::{formula_ui, sort_ui, split_cells_ui, table_to_text_ui, text_to_table_ui};
#[cfg(test)]
use text::language_name;
pub use text::{HyphenationForm, LANGUAGES, LanguageForm, LineNumbersForm, ManualForm};
use text::{hyphenation_ui, language_ui, line_numbers_ui, manual_ui};

#[derive(Clone, Debug, Serialize)]
#[serde(tag = "form", rename_all = "camelCase")]
pub enum LayoutDialog {
    LineNumbers(LineNumbersForm),
    Hyphenation(HyphenationForm),
    ManualHyphenation(ManualForm),
    Language(LanguageForm),
    Envelopes(EnvelopeForm),
    Labels(LabelsForm),
    Formula(FormulaForm),
    Sort(SortForm),
    SplitCells(SplitCellsForm),
    TextToTable(TextToTableForm),
    TableToText(TableToTextForm),
    Size(SizeForm),
    PageNumbers(PageNumbersForm),
    Theme(ThemeForm),
}

impl LayoutDialog {
    pub fn name(&self) -> &'static str {
        match self {
            LayoutDialog::LineNumbers(_) => "lineNumbers",
            LayoutDialog::Hyphenation(_) => "hyphenation",
            LayoutDialog::ManualHyphenation(_) => "manualHyphenation",
            LayoutDialog::Language(_) => "language",
            LayoutDialog::Envelopes(_) => "envelopes",
            LayoutDialog::Labels(_) => "labels",
            LayoutDialog::Formula(_) => "tableFormula",
            LayoutDialog::Sort(_) => "tableSort",
            LayoutDialog::SplitCells(_) => "splitCells",
            LayoutDialog::TextToTable(_) => "textToTable",
            LayoutDialog::TableToText(_) => "tableToText",
            LayoutDialog::Size(_) => "pictureSize",
            LayoutDialog::PageNumbers(_) => "pageNumbers",
            LayoutDialog::Theme(_) => "createTheme",
        }
    }

    /// The window title (English; the caller translates it).
    pub fn title(&self) -> &'static str {
        match self {
            LayoutDialog::LineNumbers(_) => "Line Numbers",
            LayoutDialog::Hyphenation(_) => "Hyphenation",
            LayoutDialog::ManualHyphenation(_) => "Manual Hyphenation",
            LayoutDialog::Language(_) => "Language",
            LayoutDialog::Envelopes(_) => "Envelopes",
            LayoutDialog::Labels(_) => "Labels",
            LayoutDialog::Formula(_) => "Formula",
            LayoutDialog::Sort(_) => "Sort",
            LayoutDialog::SplitCells(_) => "Split Cells",
            LayoutDialog::TextToTable(_) => "Convert Text to Table",
            LayoutDialog::TableToText(_) => "Convert to Text",
            LayoutDialog::Size(_) => "Size",
            LayoutDialog::PageNumbers(_) => "Page Number Format",
            LayoutDialog::Theme(_) => "Create New Theme",
        }
    }
}

/// The dialog `ui.dialog` opens by `name`.
pub fn open(name: &str, app: &mut WordApp) -> Option<LayoutDialog> {
    Some(match name {
        "lineNumbers" => LayoutDialog::LineNumbers(LineNumbersForm::read(app)),
        "hyphenation" => LayoutDialog::Hyphenation(HyphenationForm::read(app)),
        "manualHyphenation" => LayoutDialog::ManualHyphenation(ManualForm::start(app)),
        "language" => LayoutDialog::Language(LanguageForm::read(app)),
        "envelopes" => LayoutDialog::Envelopes(EnvelopeForm::read(app)),
        "labels" => LayoutDialog::Labels(LabelsForm::read(app)),
        "tableFormula" => LayoutDialog::Formula(FormulaForm::read(app)?),
        "tableSort" => LayoutDialog::Sort(SortForm::read(app)?),
        "splitCells" => LayoutDialog::SplitCells(SplitCellsForm::read(app)?),
        "textToTable" => LayoutDialog::TextToTable(TextToTableForm::read(app)?),
        "tableToText" => LayoutDialog::TableToText(TableToTextForm::read(app)?),
        "pictureSize" => LayoutDialog::Size(SizeForm::read(app)?),
        "pageNumbers" => LayoutDialog::PageNumbers(PageNumbersForm::read(app)),
        "createTheme" => LayoutDialog::Theme(ThemeForm::read(app)),
        _ => return None,
    })
}

/// Draws the dialog; returns true to close it.
pub fn body(app: &mut WordApp, ui: &mut Ui, d: &mut LayoutDialog) -> bool {
    match d {
        LayoutDialog::LineNumbers(f) => line_numbers_ui(app, ui, f),
        LayoutDialog::Hyphenation(f) => hyphenation_ui(app, ui, f),
        LayoutDialog::ManualHyphenation(f) => manual_ui(app, ui, f),
        LayoutDialog::Language(f) => language_ui(app, ui, f),
        LayoutDialog::Envelopes(f) => envelopes_ui(app, ui, f),
        LayoutDialog::Labels(f) => labels_ui(app, ui, f),
        LayoutDialog::Formula(f) => formula_ui(app, ui, f),
        LayoutDialog::Sort(f) => sort_ui(app, ui, f),
        LayoutDialog::SplitCells(f) => split_cells_ui(app, ui, f),
        LayoutDialog::TextToTable(f) => text_to_table_ui(app, ui, f),
        LayoutDialog::TableToText(f) => table_to_text_ui(app, ui, f),
        LayoutDialog::Size(f) => size_ui(app, ui, f),
        LayoutDialog::PageNumbers(f) => page_numbers_ui(app, ui, f),
        LayoutDialog::Theme(f) => theme_ui(app, ui, f),
    }
}

/// The accent button and Cancel; returns (ok, cancel).
fn buttons(ui: &mut Ui, ok: &str, ok_enabled: bool) -> (bool, bool) {
    let mut r = (false, false);
    ui.add_space(8.0);
    ui.horizontal(|ui| {
        ui.with_layout(crate::rtl::far_side(ui, egui::Align::Center), |ui| {
            let (okb, cancelb) =
                crate::dialogs::ok_cancel_responses(ui, |ui| ui.add_enabled(ok_enabled, crate::widgets::primary_button(ui, tl!(ok))), tl!("Cancel"));
            if cancelb.clicked() || crate::dialogs::cancel_key_pressed(ui) {
                r.1 = true;
            }
            if okb.clicked() {
                r.0 = true;
            }
        });
    });
    r
}

fn heading(ui: &mut Ui, s: &str) {
    ui.add_space(4.0);
    ui.label(egui::RichText::new(tl!(s)).font(semibold(12.5)));
}

/// Runs a dialog's command; a refusal shows in the status bar and keeps the dialog open.
fn run(app: &mut WordApp, id: &str, params: Value) -> bool {
    match app.run(id, params) {
        Ok(_) => true,
        Err(e) => {
            app.status(e);
            false
        }
    }
}

fn unit() -> wordcraft_geom::Unit {
    crate::options::unit()
}

#[cfg(test)]
mod tests;

// ---------------------------------------------------------------------------------------------
// Create New Theme

/// The Create New Theme dialog's fields: the document's current theme to start from. `colors`
/// are hex: Text/Background – Dark 2, Light 2, then Accent 1–6.
#[derive(Clone, Debug, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ThemeForm {
    pub name: String,
    pub heading: String,
    pub body: String,
    pub colors: [String; 8],
}

impl ThemeForm {
    pub fn read(app: &WordApp) -> ThemeForm {
        let s = &app.session;
        let theme = &s.doc.settings.theme_colors;
        let slot = |i: usize| theme.get(i).copied().unwrap_or_default().hex();
        // The document's own theme when it's the user's (to change it), else a new name.
        let current = &s.doc.settings.theme_name;
        let name = if s.prefs.themes.iter().any(|t| t.name.eq_ignore_ascii_case(current)) {
            current.clone()
        } else {
            let custom = tl!("Custom");
            (1..).map(|n| format!("{custom} {n}")).find(|n| !s.prefs.themes.iter().any(|t| t.name.eq_ignore_ascii_case(n))).unwrap_or_default()
        };
        ThemeForm {
            name,
            heading: s.doc.settings.major_font.clone(),
            body: s.doc.settings.minor_font.clone(),
            colors: [2, 3, 4, 5, 6, 7, 8, 9].map(slot),
        }
    }

    /// `design.createTheme` parameters for Save.
    pub fn params(&self) -> Value {
        let [text, background, accents @ ..] = &self.colors;
        json!({"name": self.name, "heading": self.heading, "body": self.body, "accents": accents, "text": text, "background": background})
    }
}

fn theme_ui(app: &mut WordApp, ui: &mut Ui, f: &mut ThemeForm) -> bool {
    let palette = app.session.doc.settings.theme_colors.clone();
    egui::Grid::new("create_theme").num_columns(2).spacing(vec2(12.0, 6.0)).show(ui, |ui| {
        ui.label(tl!("Name:"));
        ui.add(egui::TextEdit::singleline(&mut f.name).desired_width(220.0));
        ui.end_row();
        for (label, font, id) in [("Heading font:", &mut f.heading, "theme_heading"), ("Body font:", &mut f.body, "theme_body")] {
            ui.label(tl!(label));
            let shown = app.previews.font_shown(font);
            let fams = app.previews.families();
            let prev = app.previews.font_preview_fn();
            if let Some(picked) = crate::widgets::font_combo(ui, id, 220.0, &shown, &fams, Some(&*prev)) {
                if picked == crate::widgets::ADD_FONT {
                    let _ = app.run("font.add", json!({}));
                } else {
                    *font = app.previews.font_stored(&picked);
                }
            }
            ui.end_row();
        }
        for (i, value) in f.colors.iter_mut().enumerate() {
            let label = match i {
                0 => tl!("Text:").to_string(),
                1 => tl!("Background:").to_string(),
                n => crate::i18n::fmt(tl!("Accent {n}:"), &[("n", &(n - 1).to_string())]),
            };
            ui.label(label);
            // Each field its own id: the More Colors… answer must come back to the right one.
            ui.push_id(i, |ui| crate::dialogs::chrome::color_field(ui, &palette, value, "Automatic", ""));
            ui.end_row();
        }
    });
    // Preview: the colours as the Themes menu will show them.
    ui.add_space(6.0);
    ui.horizontal(|ui| {
        for c in f.colors.iter().filter_map(|h| wordcraft_doc::Rgb::parse(h)) {
            let (r, _) = ui.allocate_exact_size(vec2(22.0, 22.0), egui::Sense::hover());
            ui.painter().rect_filled(r, 3.0, crate::theme::c32(c));
            ui.painter().rect_stroke(r, 3.0, egui::Stroke::new(0.5, crate::theme::Tokens::get(ui.ctx()).border_strong), egui::StrokeKind::Inside);
        }
    });
    let name = f.name.trim().to_string();
    let saved = app.session.prefs.themes.iter().any(|t| t.name.eq_ignore_ascii_case(&name));
    if saved {
        ui.add_space(4.0);
        if ui.button(tl!("Delete Theme")).clicked() {
            return run(app, "design.deleteTheme", json!({"name": name}));
        }
    }
    let (ok, cancel) = buttons(ui, "Save", !name.is_empty());
    if ok {
        return run(app, "design.createTheme", f.params());
    }
    cancel
}
