use hex_color::HexColor;
use serde::{Serialize, Deserialize};
use slint::Color;
use directories::ProjectDirs;
use walkdir::WalkDir;
use std::{
    fs::{File, OpenOptions},
    io::{BufReader, BufWriter},
    path::Path,
    sync::OnceLock
};

use crate::JsonTheme;

#[derive(Clone, Deserialize)]
struct ThemeColors {
    #[serde(rename = "--focus-round")]
    focus_round: HexColor,
    
    #[serde(rename = "--short-break")]
    short_break: HexColor,
    
    #[serde(rename = "--long-break")]
    long_break: HexColor,

    #[serde(rename = "--background")]
    background: HexColor,

    #[serde(rename = "--background-light")]
    background_light: HexColor,

    #[serde(rename = "--background-lightest")]
    background_lightest: HexColor,

    #[serde(rename = "--foreground")]
    foreground: HexColor,

    #[serde(rename = "--foreground-darker")]
    foreground_darker: HexColor ,

    #[serde(rename = "--foreground-darkest")]
    foreground_darkest: HexColor,

    #[serde(rename = "--accent")]
    accent: HexColor,

    #[serde(rename = "--nav")]
    nav: HexColor,

    #[serde(rename = "--text-clr-primary")]
    text_clr_primary: HexColor,

    #[serde(rename = "--text-clr-secondary")]
    text_clr_secondary: HexColor
}

#[derive(Clone, Deserialize)]
pub struct JsonThemeTemplate {
    name: String,
    colors: ThemeColors
}

impl Into<JsonTheme> for JsonThemeTemplate {
    fn into(self) -> JsonTheme {
        JsonTheme {
            name: self.name.into(),
            long_break: Color::from_rgb_u8(
                self.colors.long_break.r,
                self.colors.long_break.g,
                self.colors.long_break.b,
            ).into(),
            short_break: Color::from_rgb_u8(
                self.colors.short_break.r,
                self.colors.short_break.g,
                self.colors.short_break.b,
            ).into(),
            focus_round: Color::from_rgb_u8(
                self.colors.focus_round.r,
                self.colors.focus_round.g,
                self.colors.focus_round.b,
            ).into(),
            background: Color::from_rgb_u8(
                self.colors.background.r,
                self.colors.background.g,
                self.colors.background.b,
            ).into(),
            background_light: Color::from_rgb_u8(
                self.colors.background_light.r,
                self.colors.background_light.g,
                self.colors.background_light.b,
            ).into(),
            background_lightest: Color::from_rgb_u8(
                self.colors.background_lightest.r,
                self.colors.background_lightest.g,
                self.colors.background_lightest.b,
            ).into(),
            foreground: Color::from_rgb_u8(
                self.colors.foreground.r,
                self.colors.foreground.g,
                self.colors.foreground.b,
            ).into(),
            foreground_darker: Color::from_rgb_u8(
                self.colors.foreground_darker.r,
                self.colors.foreground_darker.g,
                self.colors.foreground_darker.b,
            ).into(),
            foreground_darkest: Color::from_rgb_u8(
                self.colors.foreground_darkest.r,
                self.colors.foreground_darkest.g,
                self.colors.foreground_darkest.b,
            ).into(),
            accent: Color::from_rgb_u8(
                self.colors.accent.r,
                self.colors.accent.g,
                self.colors.accent.b,
            ).into(),
            nav: Color::from_rgb_u8(
                self.colors.nav.r,
                self.colors.nav.g,
                self.colors.nav.b,
            ).into(),
            text_clr_primary: Color::from_rgb_u8(
                self.colors.text_clr_primary.r,
                self.colors.text_clr_primary.g,
                self.colors.text_clr_primary.b,
            ).into(),
            text_clr_secondary: Color::from_rgb_u8(
                self.colors.text_clr_secondary.r,
                self.colors.text_clr_secondary.g,
                self.colors.text_clr_secondary.b,
            ).into()
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct JsonSettings {
    pub always_on_top: bool,
    pub auto_start_focus_timer: bool,
    pub auto_start_break_timer: bool,
    pub minimize_to_tray: bool,
    pub minimize_to_tray_on_close: bool,
    pub notifications: bool,
    pub focus: i32,
    pub short_break: i32,
    pub long_break: i32,
    pub rounds: i32,
    pub ambient_volume: i32,
    pub alerts_volume: i32,
    pub theme: String,
    pub song: String
}

static CONFIG_DIR: OnceLock<Option<ProjectDirs>> = OnceLock::new();
static DEFAULT_THEME: OnceLock<JsonThemeTemplate> = OnceLock::new();
static DEFAULT_SONG: OnceLock<String> = OnceLock::new();

fn get_dir() -> Option<&'static Path> {
    if let Some(dirs) = CONFIG_DIR.get_or_init(|| ProjectDirs::from("io", "Risuleia", "Tranquilo")) {
        Some(dirs.config_dir())
    } else {
        None
    }
}

pub fn default_theme() -> &'static JsonThemeTemplate {
    DEFAULT_THEME.get_or_init(|| {
        let default_theme = r##"{
            "name": "Eclipse",
            "colors": {
                "--long-break": "#af486d",
                "--short-break": "#c3cffb",
                "--focus-round": "#eebc7f",
                "--background": "#0c0912",
                "--background-light": "#343132",
                "--background-lightest": "#837c7e",
                "--foreground": "#dfdfd7",
                "--foreground-darker": "#bec0c0",
                "--foreground-darkest": "#adadae",
                "--accent": "#7147f0",
                "--nav": "#1d2545",
                "--text-clr-primary": "#b4a3f8",
                "--text-clr-secondary": "#cbc5db"
            }
        }"##;
        serde_json::from_str::<JsonThemeTemplate>(default_theme).unwrap()
    })
}

pub fn default_song() -> &'static String {
    DEFAULT_SONG.get_or_init(|| "Colorful Flowers".to_string())
}

pub fn load_settings() -> JsonSettings {
    if let Some(config_dir) = get_dir() {
        let file = config_dir.join("config.json");
        if let Ok(set_file) = File::open(file) {
            let reader = BufReader::new(set_file);
            serde_json::from_reader(reader).expect("Loading settings from JSON")
        } else {
            default_settings()
        }
    } else {
        default_settings()
    }
}

pub fn load_themes() -> Vec<JsonTheme> {
    let theme_dir = Path::new("assets/themes");

    let mut themes: Vec<JsonTheme> = WalkDir::new(theme_dir)
        .into_iter()
        .filter(|r| {
            r.as_ref().map_or(false, |f| {
                f.file_name()
                    .to_str()
                    .map(|s| s.to_lowercase().ends_with(".json"))
                    .unwrap_or(false)
            })
        })
        .filter_map(|r| {
            r.map(|d| {
                let reader = BufReader::new(File::open(d.path()).unwrap());
                let theme = std::io::read_to_string(reader).unwrap();
                serde_json::from_str::<JsonThemeTemplate>(&theme)
                    .unwrap()
                    .into()
            })
            .ok()
        })
        .collect();

    if themes.is_empty() {
        themes.push((*default_theme()).clone().into())
    }

    themes.sort_by(|a, b| a.name.partial_cmp(&b.name).unwrap());
    themes
}

pub fn load_songs() -> Vec<String> {
    let music_dir = Path::new("assets/music");

    let mut songs: Vec<String> = WalkDir::new(music_dir)
        .into_iter()
        .filter(|r| {
            r.as_ref().map_or(false, |f| {
                f.file_name()
                    .to_str()
                    .map(|s| s.to_lowercase().ends_with(".mp3"))
                    .unwrap_or(false)
            })
        })
        .filter_map(|r| {
            r.map(|d| {
                let song_name = d.file_name().to_str().map(|s| s.trim_end_matches(".mp3")).unwrap().to_string();
                song_name
            })
            .ok()
        })
        .collect();

    if songs.is_empty() {
        songs.push((*default_song()).clone().into())
    }

    songs.sort_by(|a, b| a.partial_cmp(&b).unwrap());
    songs
}

fn default_settings() -> JsonSettings {
    let def_settings = include_bytes!("../assets/default-preferences.json");
    serde_json::from_reader(&def_settings[..]).unwrap()
}

pub fn save_settings(settings: JsonSettings) {
    if let Some(config_dir) = get_dir() {
        std::fs::create_dir_all(config_dir).unwrap();

        let file = config_dir.join("config.json");
        let set_file = OpenOptions::new()
            .write(true)
            .create(true)
            .truncate(true)
            .open(file)
            .unwrap();
        
        let writer = BufWriter::new(set_file);
        serde_json::to_writer_pretty(writer, &settings).expect("Writing settings to JSON");
    }
}


    // let theme_dir = {
    //     let mut theme_dir = PathBuf::from("get_dir().unwrap()");
    //     theme_dir.push("themes");
    //     theme_dir
    // };