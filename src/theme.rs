use ratatui::style::Color;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Theme {
    pub name: String,
    #[serde(with = "color_serde")]
    pub bg: Color,
    #[serde(with = "color_serde")]
    pub panel_bg: Color,
    #[serde(with = "color_serde")]
    pub magenta: Color,
    #[serde(with = "color_serde")]
    pub cyan: Color,
    #[serde(with = "color_serde")]
    pub pink: Color,
    #[serde(with = "color_serde")]
    pub yellow: Color,
    #[serde(with = "color_serde")]
    pub orange: Color,
    #[serde(with = "color_serde")]
    pub white: Color,
    #[serde(with = "color_serde")]
    pub gray: Color,
    #[serde(with = "color_serde")]
    pub green: Color,
    #[serde(with = "color_serde")]
    pub red: Color,
    #[serde(with = "color_serde")]
    pub border: Color,
    #[serde(with = "color_serde")]
    pub highlight: Color,
    #[serde(with = "color_serde")]
    pub git_added: Color,
    #[serde(with = "color_serde")]
    pub git_modified: Color,
    #[serde(with = "color_serde")]
    pub git_untracked: Color,
}

impl Default for Theme {
    fn default() -> Self {
        Self::synthwave()
    }
}

impl Theme {
    pub fn synthwave() -> Self {
        Self {
            name: "synthwave".to_string(),
            bg: Color::Rgb(10, 2, 26),
            panel_bg: Color::Rgb(18, 4, 42),
            magenta: Color::Rgb(255, 0, 255),
            cyan: Color::Rgb(0, 255, 255),
            pink: Color::Rgb(255, 20, 147),
            yellow: Color::Rgb(255, 215, 0),
            orange: Color::Rgb(255, 140, 0),
            white: Color::Rgb(240, 240, 255),
            gray: Color::Rgb(120, 120, 140),
            green: Color::Rgb(57, 255, 20),
            red: Color::Rgb(255, 50, 80),
            border: Color::Rgb(180, 0, 180),
            highlight: Color::Rgb(40, 0, 80),
            git_added: Color::Rgb(57, 255, 20),
            git_modified: Color::Rgb(255, 215, 0),
            git_untracked: Color::Rgb(255, 50, 80),
        }
    }

    pub fn load(name: &str) -> Self {
        match name {
            "synthwave" => Self::synthwave(),
            _ => Self::synthwave(),
        }
    }
}

mod color_serde {
    use ratatui::style::Color;
    use serde::{self, Deserialize, Deserializer, Serializer};

    pub fn serialize<S>(color: &Color, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let s = match color {
            Color::Rgb(r, g, b) => format!("#{:02x}{:02x}{:02x}", r, g, b),
            _ => "#000000".to_string(),
        };
        serializer.serialize_str(&s)
    }

    pub fn deserialize<'de, D>(deserializer: D) -> Result<Color, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        if s.starts_with('#') && s.len() == 7 {
            let r = u8::from_str_radix(&s[1..3], 16).unwrap_or(0);
            let g = u8::from_str_radix(&s[3..5], 16).unwrap_or(0);
            let b = u8::from_str_radix(&s[5..7], 16).unwrap_or(0);
            Ok(Color::Rgb(r, g, b))
        } else {
            Ok(Color::Black)
        }
    }
}
