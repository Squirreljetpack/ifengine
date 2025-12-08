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
        if path.is_dir() && !path.file_name().and_then(|f| f.to_str()).and_then(|f| Some(f.starts_with('_'))).unwrap_or(true) {
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
                        let name = path.file_stem().unwrap().to_str().unwrap();
                        let name = if name.len() > 2 && name.chars().nth(0).unwrap().is_ascii_digit() && name.chars().nth(1).unwrap() == '-' {
                            &name[2..]
                        } else {
                            name
                        };

                        let absolute_path = path.to_str().unwrap();

                        font_definitions_code.push_str(&format!(
                            "    fonts.font_data.insert(\"{}\".to_owned(), egui::FontData::from_static(include_bytes!(\"{}\")).into());\n",
                            name, absolute_path
                        ));

                        font_names.push(name.to_string());
                    }
                }
            }
        }

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

    // (Rust Name, VS Code keys, token scopes)
    let mappings: Vec<(&str, Vec<&str>, Vec<&str>)> = vec![
    ("RED", vec!["terminal.ansiRed", "errorForeground", "editorError.foreground"], vec!["token.error-token"]),
    ("YELLOW", vec!["terminal.ansiYellow", "editorWarning.foreground"], vec!["token.warn-token"]),
    ("GREEN", vec!["terminal.ansiBrightGreen", "terminal.ansiGreen", "gitDecoration.untrackedResourceForeground"], vec!["string"]),
    ("BLUE", vec!["terminal.ansiBlue", "textLink.activeForeground"], vec!["token.info-token"]),
    ("PURPLE", vec!["terminal.ansiBrightMagenta", "terminal.ansiMagenta"], vec!["token.debug-token"]),
    ("BLACK", vec!["terminal.ansiBlack", "editor.background"], vec![]),
    ("WHITE", vec!["terminal.ansiWhite", "editor.foreground"], vec![]),
    ("GRAY", vec!["editorCodeLens.foreground", "descriptionForeground", "scrollbar.shadow"], vec![]),
    ("DARK_GRAY", vec!["sideBar.border", "activityBar.border", "panel.border", "badge.background", "statusBarItem.remoteBackground", "titleBar.inactiveForeground", "tab.inactiveForeground"], vec![]),
    ("LIGHT_GRAY", vec!["tab.inactiveForeground", "terminal.ansiBrightBlack"], vec![]),
    // Logical
    ("BG", vec!["editor.background", "background"], vec![]),
    ("BG_SECONDARY", vec!["sideBar.background", "editorWidget.background"], vec![]),
    ("SELECTION_FG", vec!["list.activeSelectionForeground", "editor.foreground"], vec![]),
    ("SELECTION_BG", vec!["editor.selectionBackground", "list.activeSelectionBackground", "menu.selectionBackground", "menu.selectionHighlightBackground"], vec![]),
    ("MUTED2", vec!["scrollbarSlider.background", "editorGuide.background"], vec![]),
    ("STRONG", vec!["editor.foreground"], vec![]), // not sure how to set it apart
    ("PRIMARY", vec![], vec!["variable", "meta.function-call.arguments"]),
    ("MUTED", vec![], vec!["comment"]),
    ("SECONDARY", vec![], vec!["constant", "constant.language"]),
    ("STRING", vec![], vec!["string", "meta.preprocessor.string"]),
    ("BORDER", vec!["panel.border", "menu.border", "dropdown.border", "editorGroup.border", "sideBar.border", "focusBorder"], vec![]),
    ];

    // ------- BEGIN OUTPUT --------------------
    output.push_str(&format!("pub mod {variant} {{\n"));
    output.push_str("    use egui::{Color32, hex_color};\n");

    // --------  generate COLORS_MAP
    let mut map_output = String::new();
    map_output.push_str("    use phf::{self, phf_map};\n");
    map_output.push_str("    pub static COLORS_MAP: phf::Map<&'static str, Color32> = phf_map! {\n");

    // -------- Find colors based on priority and output ----------------
    for (name, keys, scopes) in mappings {
        let hex = resolve_color(&theme.colors, &keys)
        .or_else(|| resolve_token_color(&theme, &scopes))
        .or_else(|| if name == "PRIMARY" {
            resolve_color(&theme.colors, &vec!["editor.foreground"])
        } else { None });



        if let Some(hex) = hex {
            output.push_str(&format!("    pub const {}: Color32 = hex_color!(\"{}\");\n", name, hex));
            map_output.push_str(&format!("    \"{}\" => {},\n", name.to_lowercase(), name));
        }
    }

    map_output.push_str("    };\n");
    output.push_str(&map_output);
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
