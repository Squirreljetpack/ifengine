use std::collections::HashMap;
use std::env;
use std::fs;
use std::fs::File;
use std::io::Write;
use std::path::Path;
use std::path::PathBuf;

use serde::Deserialize;
use serde_hjson::Value;
use serde_hjson::from_str;

const FONTS_PATH: &str = "../assets/fonts";
const COLORS_PATH: &str = "../assets/colors";

fn to_absolute(p: impl AsRef<Path>) -> PathBuf {
    Path::new(&env::var("CARGO_MANIFEST_DIR").unwrap()).join(p.as_ref())
}
fn out_file(p: impl AsRef<Path>) -> File {
    let dest = Path::new(&env::var_os("OUT_DIR").unwrap()).join(p.as_ref());
    fs::File::create(&dest).unwrap()
}

fn main() {
    println!("cargo:rerun-if-changed=build.rs");
    // fonts
    println!("cargo:rerun-if-changed={FONTS_PATH}");
    let mut out = out_file("generated_fonts.rs");

    let mut code = String::new();
    code.push_str("pub fn add_fonts(fonts: &mut egui::FontDefinitions) {\n");

    for entry in fs::read_dir(&to_absolute(FONTS_PATH)).unwrap() {
        let entry = entry.unwrap();
        let path = entry.path();
        if path.is_dir() {
            let family_name = path.file_name().unwrap().to_str().unwrap();
            add_fonts_from_dir(
                &mut code,
                &to_absolute(&format!("{FONTS_PATH}/{}", family_name)),
                family_name,
            );
        }
    }
    code.push_str("}\n");
    write!(out, "{}", code).unwrap();

    println!("cargo:rerun-if-changed={COLORS_PATH}");
    let mut out = out_file("generated_colors.rs");
    let mut code = String::new();
    for entry in fs::read_dir(to_absolute(COLORS_PATH)).unwrap() {
        let entry = entry.unwrap();
        let path = entry.path();
        if let Some(stem) = path.file_stem() {
            let basename = stem.to_string_lossy();
            add_colors_from_theme(&mut code, &path, &basename);
        }
    }
    write!(out, "{}", code).unwrap();
}

fn add_fonts_from_dir(
    font_definitions_code: &mut String,
    fonts_dir: &Path,
    font_family_name: &str,
) {
    if fonts_dir.is_dir() {
        let mut font_names = Vec::new();

        for entry in fs::read_dir(fonts_dir).unwrap() {
            let entry = entry.unwrap();
            let path = entry.path();
            if path.is_file() {
                if let Some(ext) = path.extension() {
                    if ext == "ttf" || ext == "otf" {
                        let font_name = path.file_stem().unwrap().to_str().unwrap();
                        let absolute_path = path.to_str().unwrap();

                        font_definitions_code.push_str(&format!(
                            "    fonts.font_data.insert(\"{}\".to_owned(), egui::FontData::from_static(include_bytes!(\"{}\")).into());\n",
                            font_name, absolute_path
                        ));

                        font_names.push(font_name.to_string());
                    }
                }
            }
        }

        font_names.sort(); // strip variant suffix from default font to put it first

        let egui_family = match font_family_name.to_lowercase().as_str() {
            "proportional" => "egui::FontFamily::Proportional".to_string(),
            "monospace" => "egui::FontFamily::Monospace".to_string(),
            other => format!("egui::FontFamily::Name(\"{}\".into())", other),
        };

        let font_array = font_names
            .iter()
            .map(|n| format!("\"{}\".to_owned()", n))
            .collect::<Vec<_>>()
            .join(", ");

        font_definitions_code.push_str(&format!(
            "    fonts.families.insert({}, vec![{}]);\n",
            egui_family, font_array
        ));
    }
}

#[derive(Deserialize, Debug)]
struct VsCodeTheme {
    colors: HashMap<String, String>,
    #[serde(rename = "tokenColors")]
    token_colors: Option<Vec<TokenColor>>,
}

#[derive(Deserialize, Debug)]
struct TokenColor {
    scope: Option<Value>, // Can be string or array of strings
    settings: TokenSettings,
}

#[derive(Deserialize, Debug)]
struct TokenSettings {
    foreground: Option<String>,
}

fn add_colors_from_theme(output: &mut String, theme_file: &Path, variant: &str) {
    let content = fs::read_to_string(theme_file).expect("Could not read theme file");
    let theme: VsCodeTheme = from_str(&content).expect("Failed to parse theme JSON");

    // Define our mapping logic
    // Format: (Rust Constant Name, List of VS Code Keys to try in order)
    let mappings = vec![
        (
            "RED",
            vec![
                "terminal.ansiRed",
                "errorForeground",
                "editorError.foreground",
            ],
        ),
        (
            "YELLOW",
            vec!["terminal.ansiYellow", "editorWarning.foreground"],
        ),
        (
            "GREEN",
            vec![
                "terminal.ansiBrightGreen",
                "terminal.ansiGreen",
                "gitDecoration.untrackedResourceForeground",
            ],
        ),
        (
            "BLUE",
            vec!["terminal.ansiBlue", "textLink.activeForeground"],
        ),
        (
            "PURPLE",
            vec!["terminal.ansiBrightMagenta", "terminal.ansiMagenta"],
        ),
        ("BLACK", vec!["terminal.ansiBlack", "editor.background"]),
        ("WHITE", vec!["terminal.ansiWhite", "editor.foreground"]),
        // random grays?
        (
            "GRAY",
            vec![
                "editorCodeLens.foreground",
                "descriptionForeground",
                "scrollbar.shadow",
            ],
        ),
        (
            "DARK_GRAY",
            vec![
                // Your originals (UI element borders)
                "sideBar.border",
                "activityBar.border",
                "panel.border",
                // Badge / label grays
                "badge.background",
                "statusBarItem.remoteBackground",
                // Tab / window UI
                "titleBar.inactiveForeground",
                "tab.inactiveForeground",
            ],
        ),
        (
            "LIGHT_GRAY",
            vec!["tab.inactiveForeground", "terminal.ansiBrightBlack"],
        ),
        // Logical
        ("BG", vec!["editor.background", "background"]),
        (
            "BG_SECONDARY",
            vec!["sideBar.background", "editorWidget.background"],
        ),
        (
            "SELECTION_FG",
            vec!["list.activeSelectionForeground", "editor.foreground"],
        ),
        (
            "SELECTION_BG",
            vec![
                "editor.selectionBackground",
                "list.activeSelectionBackground",
                "menu.selectionBackground",
                "menu.selectionHighlightBackground",
            ],
        ),
        // again, no clear correspondence
        (
            "MUTED2",
            vec!["scrollbarSlider.background", "editorGuide.background"],
        ),
        ("STRONG", vec!["editor.foreground"]),
    ];

    let scoped_mappings = vec![
        ("PRIMARY", vec!["variable"]),
        ("MUTED", vec!["comment"]),
        ("SECONDARY", vec!["constant"]),
        ("STRING", vec!["string", "meta.preprocessor.string"]),
    ];

    // ------- BEGIN OUTPUT --------------------
    output.push_str(&format!("pub mod {variant} {{\n"));

    // --------  generate COLORS_MAP, though you may not want to import this
    // output.push_str("    use phf::{self, phf_map};\n");

    // output.push_str("    pub static COLORS_MAP: phf::Map<&'static str, Color32> = phf_map! {\n");

    // for (name, _) in &mappings {
    //     output.push_str(&format!("    \"{}\" => {},\n", name.to_lowercase(), name));
    // }
    // for (name, _) in &scoped_mappings {
    //     output.push_str(&format!("    \"{}\" => {},\n", name.to_lowercase(), name));
    // }
    // output.push_str("    };\n");

    // -------- Find colors based on priority and output ----------------
    output.push_str("    use egui::{Color32, hex_color};\n");
    for (name, keys) in mappings {
        if let Some(hex) = resolve_color(&theme.colors, &keys) {
            output.push_str(&format!(
                "    pub const {}: Color32 = hex_color!(\"{}\");\n",
                name, hex
            ));
        }
    }

    // generate constants for scoped mappings
    for (name, scope) in scoped_mappings {
        if let Some(hex) = resolve_token_color(&theme, &scope) {
            output.push_str(&format!(
                "    pub const {}: Color32 = hex_color!(\"{}\");\n",
                name, hex
            ));
        }
    }
    output.push_str("}\n");
}

/// Helper to find first matching key in the map
fn resolve_color<'a>(map: &'a HashMap<String, String>, keys: &[&str]) -> Option<&'a str> {
    keys.iter()
        .find_map(|&key| map.get(key).map(|val| val.as_str()))
}

fn resolve_token_color<'a>(theme: &'a VsCodeTheme, target_scopes: &[&str]) -> Option<&'a str> {
    theme.token_colors.as_ref()?.iter().find_map(|token| {
        let fg = token.settings.foreground.as_ref()?;
        match &token.scope {
            Some(Value::String(s)) => {
                let mut arr = s.split(',');
                arr.find_map(|s| target_scopes.contains(&s.trim()).then_some(fg.as_str()))
            }
            Some(Value::Array(arr)) => arr.iter().find_map(|item| {
                item.as_str()
                    .and_then(|s| (target_scopes.contains(&s)).then_some(fg.as_str()))
            }),
            _ => None,
        }
    })
}
