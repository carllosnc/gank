use eframe::egui::{Color32, FontId, Margin, Rounding, Vec2};

/// Paleta oficial Tailwind Blue e Slate
#[allow(dead_code)]
pub mod colors {
    use super::Color32;

    // --- Tailwind Blue Scale ---
    pub const BLUE_50: Color32 = Color32::from_rgb(239, 246, 255);
    pub const BLUE_100: Color32 = Color32::from_rgb(219, 234, 254);
    pub const BLUE_200: Color32 = Color32::from_rgb(191, 219, 254);
    pub const BLUE_300: Color32 = Color32::from_rgb(147, 197, 253);
    pub const BLUE_400: Color32 = Color32::from_rgb(96, 165, 250);
    pub const BLUE_500: Color32 = Color32::from_rgb(59, 130, 246); // Acento Primário
    pub const BLUE_600: Color32 = Color32::from_rgb(37, 99, 235); // Hover Primário
    pub const BLUE_700: Color32 = Color32::from_rgb(29, 78, 216); // Active Primário
    pub const BLUE_800: Color32 = Color32::from_rgb(30, 64, 175);
    pub const BLUE_900: Color32 = Color32::from_rgb(30, 58, 138);
    pub const BLUE_950: Color32 = Color32::from_rgb(23, 37, 84);

    // --- Tailwind Slate Scale (Superfícies & Neutros) ---
    pub const SLATE_50: Color32 = Color32::from_rgb(248, 250, 252);
    pub const SLATE_100: Color32 = Color32::from_rgb(241, 245, 249);
    pub const SLATE_200: Color32 = Color32::from_rgb(226, 232, 240);
    pub const SLATE_300: Color32 = Color32::from_rgb(203, 213, 225);
    pub const SLATE_400: Color32 = Color32::from_rgb(148, 163, 184);
    pub const SLATE_500: Color32 = Color32::from_rgb(100, 116, 139);
    pub const SLATE_600: Color32 = Color32::from_rgb(71, 85, 105);
    pub const SLATE_700: Color32 = Color32::from_rgb(51, 65, 85);
    pub const SLATE_800: Color32 = Color32::from_rgb(30, 41, 59);
    pub const SLATE_900: Color32 = Color32::from_rgb(15, 23, 42);
    pub const SLATE_950: Color32 = Color32::from_rgb(2, 6, 23);

    // --- Cores Semânticas do Sistema Gank ---
    pub const ACCENT_PRIMARY: Color32 = BLUE_500;
    pub const ACCENT_HOVER: Color32 = BLUE_600;
    pub const ACCENT_ACTIVE: Color32 = BLUE_700;

    pub const TEXT_PRIMARY: Color32 = Color32::WHITE;
    pub const TEXT_SECONDARY: Color32 = SLATE_200;
    pub const TEXT_MUTED: Color32 = SLATE_400;
    pub const TEXT_DIM: Color32 = SLATE_500;

    pub const SURFACE_PANEL: Color32 = Color32::from_rgba_premultiplied(0, 0, 0, 77); // 30% Preto Translúcido
    pub const SURFACE_CARD: Color32 = Color32::from_rgb(15, 23, 42);
    pub const SURFACE_GLASS: Color32 = Color32::from_rgba_premultiplied(12, 12, 12, 12);
    pub const SURFACE_GLASS_HOVER: Color32 = Color32::from_rgba_premultiplied(22, 22, 22, 22);

    pub const BORDER_SUBTLE: Color32 = Color32::from_rgba_premultiplied(20, 20, 20, 20);
    pub const BORDER_ACCENT: Color32 = Color32::from_rgba_premultiplied(45, 77, 117, 120);

    // Feedback
    pub const SUCCESS: Color32 = Color32::from_rgb(52, 211, 153);
    pub const WARNING: Color32 = Color32::from_rgb(251, 146, 60);
    pub const ERROR: Color32 = Color32::from_rgb(248, 113, 113);
}

/// Escala Tipográfica Oficial Gank
#[allow(dead_code)]
pub mod typography {
    use super::FontId;

    pub fn display_large() -> FontId {
        FontId::proportional(34.0)
    }

    pub fn display_medium() -> FontId {
        FontId::proportional(26.0)
    }

    pub fn title_large() -> FontId {
        FontId::proportional(22.0)
    }

    pub fn title_medium() -> FontId {
        FontId::proportional(17.5)
    }

    pub fn body_large() -> FontId {
        FontId::proportional(16.0)
    }

    pub fn body_medium() -> FontId {
        FontId::proportional(14.5)
    }

    pub fn body_small() -> FontId {
        FontId::proportional(13.0)
    }

    pub fn caption() -> FontId {
        FontId::proportional(11.5)
    }

    pub fn monospace() -> FontId {
        FontId::monospace(13.0)
    }
}

/// Escala de Arredondamento (Border Radius)
#[allow(dead_code)]
pub mod radii {
    use super::Rounding;

    pub const SM: Rounding = Rounding::same(6.0);
    pub const MD: Rounding = Rounding::same(8.0);
    pub const LG: Rounding = Rounding::same(12.0);
    pub const FULL: Rounding = Rounding::same(999.0);
}

/// Espaçamento Padronizado & Padding de Páginas
pub mod spacing {
    use super::{Margin, Vec2};

    #[allow(dead_code)]
    pub const XS: f32 = 4.0;
    #[allow(dead_code)]
    pub const SM: f32 = 8.0;
    #[allow(dead_code)]
    pub const MD: f32 = 12.0;
    #[allow(dead_code)]
    pub const LG: f32 = 16.0;
    #[allow(dead_code)]
    pub const XL: f32 = 24.0;
    #[allow(dead_code)]
    pub const XXL: f32 = 32.0;

    /// Padding oficial unificado para todas as páginas da aplicação (X = 28.0, Y = 24.0)
    pub const PAGE_PADDING_X: f32 = 28.0;
    pub const PAGE_PADDING_Y: f32 = 24.0;

    /// Margem constante para frames de páginas (Biblioteca, Detalhes do Jogo, Design System)
    pub const PAGE_PADDING: Margin = Margin {
        left: PAGE_PADDING_X,
        right: PAGE_PADDING_X,
        top: PAGE_PADDING_Y,
        bottom: PAGE_PADDING_Y,
    };

    /// Vetor 2D constante para contração de retângulos de páginas (`.shrink2(...)`)
    #[allow(dead_code)]
    pub const PAGE_PADDING_VEC: Vec2 = Vec2::new(PAGE_PADDING_X, PAGE_PADDING_Y);
}
