use ratatui::style::Color;

/// Modern Dark Palette (Inspired by Catppuccin Mocha & Tokyo Night)
#[allow(dead_code)]
pub struct Theme {
    pub bg: Color,
    pub card_border: Color,
    pub title: Color,
    pub text_main: Color,
    pub text_muted: Color,
    pub text_dim: Color,
    
    // Accents
    pub cyan: Color,
    pub emerald: Color,
    pub violet: Color,
    pub amber: Color,
    pub rose: Color,
    pub blue: Color,
    pub peach: Color,
}

pub const THEME: Theme = Theme {
    bg: Color::Reset,
    card_border: Color::Rgb(74, 88, 122),    // Subtle slate border
    title: Color::Rgb(205, 214, 244),          // Crisp light text
    text_main: Color::Rgb(245, 245, 250),      // Pure readable white
    text_muted: Color::Rgb(147, 154, 183),     // Medium slate gray (high contrast vs black)
    text_dim: Color::Rgb(108, 112, 134),       // Subtle dim text
    
    cyan: Color::Rgb(137, 220, 235),           // Vibrant sky cyan
    emerald: Color::Rgb(166, 227, 161),        // Soft readable green
    violet: Color::Rgb(203, 166, 247),         // Soft pastel lavender/purple
    amber: Color::Rgb(249, 226, 175),          // Warm pastel gold
    rose: Color::Rgb(243, 139, 168),           // Soft red/rose for warnings/loss
    blue: Color::Rgb(137, 180, 250),           // Ice blue
    peach: Color::Rgb(250, 179, 135),          // Soft orange
};
