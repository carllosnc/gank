#![allow(dead_code)]

use crate::models::Platform;
use crate::ui::theme::{colors, radii, typography};
use eframe::egui::{self, Color32, Stroke};

/// Componente de pílula de filtro navegacional normalizado com a paleta Tailwind Blue
pub fn filter_pill(
    ui: &mut egui::Ui,
    selected: bool,
    text: &str,
    icon: Option<&str>,
) -> egui::Response {
    let font_id = typography::body_medium();

    let text_color = if selected {
        colors::TEXT_PRIMARY
    } else {
        colors::TEXT_MUTED
    };

    let mut content = String::new();
    if let Some(ic) = icon {
        content.push_str(ic);
        content.push(' ');
    }
    content.push_str(text);

    let gallery = ui.painter().layout_no_wrap(content, font_id, text_color);

    let padding_x = 14.0;
    let desired_size = egui::vec2(gallery.size().x + padding_x * 2.0, 34.0);
    let (rect, response) = ui.allocate_exact_size(desired_size, egui::Sense::click());

    if ui.is_rect_visible(rect) {
        let is_hovered = response.hovered();

        let bg_color = if selected {
            // Tailwind Blue-900 translúcido com destaque
            Color32::from_rgb(30, 58, 138)
        } else if is_hovered {
            colors::SURFACE_GLASS_HOVER
        } else {
            Color32::TRANSPARENT
        };

        let stroke = if selected {
            Stroke::NONE
        } else if is_hovered {
            Stroke::new(1.0_f32, colors::BORDER_SUBTLE)
        } else {
            Stroke::NONE
        };

        ui.painter().rect(rect, radii::FULL, bg_color, stroke);

        let text_pos = rect.center() - gallery.size() / 2.0;
        ui.painter().galley(text_pos, gallery, text_color);
    }

    if response.hovered() {
        ui.output_mut(|o| o.cursor_icon = egui::CursorIcon::PointingHand);
    }

    response
}

/// Componente de pílula de filtro com ícone de imagem oficial da plataforma
pub fn filter_pill_image(
    ui: &mut egui::Ui,
    selected: bool,
    text: &str,
    image: Option<egui::ImageSource<'static>>,
    show_text: bool,
) -> egui::Response {
    let font_id = typography::body_medium();

    let text_color = if selected {
        colors::TEXT_PRIMARY
    } else {
        colors::TEXT_MUTED
    };

    let (gallery, text_w) = if show_text {
        let g = ui
            .painter()
            .layout_no_wrap(text.to_string(), font_id, text_color);
        let w = g.size().x;
        (Some(g), w)
    } else {
        (None, 0.0)
    };

    let padding_x = if show_text { 14.0 } else { 8.0 };
    let icon_size = egui::vec2(18.0, 18.0);
    let icon_spacing = if show_text && image.is_some() {
        7.0
    } else {
        0.0
    };

    let total_content_w = if image.is_some() {
        icon_size.x + icon_spacing + text_w
    } else {
        text_w
    };

    let desired_size = if !show_text && image.is_some() {
        egui::vec2(34.0, 34.0)
    } else {
        egui::vec2(total_content_w + padding_x * 2.0, 34.0)
    };

    let (rect, mut response) = ui.allocate_exact_size(desired_size, egui::Sense::click());

    if !show_text {
        response = response.on_hover_text(text);
    }

    if ui.is_rect_visible(rect) {
        let is_hovered = response.hovered();

        let bg_color = if selected {
            // Tailwind Blue-900 translúcido com destaque
            Color32::from_rgb(30, 58, 138)
        } else if is_hovered {
            colors::SURFACE_GLASS_HOVER
        } else {
            Color32::TRANSPARENT
        };

        let stroke = if selected {
            Stroke::NONE
        } else if is_hovered {
            Stroke::new(1.0_f32, colors::BORDER_SUBTLE)
        } else {
            Stroke::NONE
        };

        ui.painter().rect(rect, radii::FULL, bg_color, stroke);

        let mut content_x = rect.center().x - total_content_w / 2.0;

        if let Some(img_source) = image {
            let icon_rect = egui::Rect::from_min_size(
                egui::pos2(content_x, rect.center().y - icon_size.y / 2.0),
                icon_size,
            );

            let tint = if selected || is_hovered {
                Color32::WHITE
            } else {
                Color32::from_white_alpha(215)
            };

            egui::Image::new(img_source)
                .fit_to_exact_size(icon_size)
                .tint(tint)
                .paint_at(ui, icon_rect);

            content_x += icon_size.x + icon_spacing;
        }

        if let Some(g) = gallery {
            let label_color = if selected || is_hovered {
                colors::TEXT_PRIMARY
            } else {
                colors::TEXT_MUTED
            };

            let text_pos = egui::pos2(content_x, rect.center().y - g.size().y / 2.0);
            ui.painter().galley(text_pos, g, label_color);
        }
    }

    if response.hovered() {
        ui.output_mut(|o| o.cursor_icon = egui::CursorIcon::PointingHand);
    }

    response
}

/// Botão de ícone quadrado arredondado com efeito de hover suave
pub fn icon_button(ui: &mut egui::Ui, icon: &str, tooltip: &str) -> egui::Response {
    let size = egui::vec2(32.0, 32.0);
    let (rect, response) = ui.allocate_exact_size(size, egui::Sense::click());

    if ui.is_rect_visible(rect) {
        let is_hovered = response.hovered();

        let bg_color = if response.is_pointer_button_down_on() {
            colors::BLUE_950
        } else if is_hovered {
            colors::SURFACE_GLASS_HOVER
        } else {
            colors::SURFACE_GLASS
        };

        let stroke = if is_hovered {
            Stroke::new(1.0_f32, colors::BLUE_400)
        } else {
            Stroke::new(1.0_f32, colors::BORDER_SUBTLE)
        };

        ui.painter().rect(rect, radii::FULL, bg_color, stroke);

        let icon_color = if is_hovered {
            colors::TEXT_PRIMARY
        } else {
            colors::TEXT_MUTED
        };

        ui.painter().text(
            rect.center(),
            egui::Align2::CENTER_CENTER,
            icon,
            typography::title_medium(),
            icon_color,
        );
    }

    if response.hovered() {
        ui.output_mut(|o| o.cursor_icon = egui::CursorIcon::PointingHand);
    }

    response.on_hover_text(tooltip)
}

/// Botão primário com preenchimento Tailwind Blue (ex: "JOGAR AGORA")
pub fn primary_button(
    ui: &mut egui::Ui,
    icon: Option<&str>,
    label: &str,
    width: Option<f32>,
) -> egui::Response {
    let text = if let Some(ic) = icon {
        format!("{}  {}", ic, label)
    } else {
        label.to_string()
    };

    let font_id = typography::body_medium();
    let gallery = ui
        .painter()
        .layout_no_wrap(text, font_id, colors::TEXT_PRIMARY);

    let w = width.unwrap_or(gallery.size().x + 36.0);
    let size = egui::vec2(w, 42.0);
    let (rect, response) = ui.allocate_exact_size(size, egui::Sense::click());

    if ui.is_rect_visible(rect) {
        let is_hovered = response.hovered();
        let is_down = response.is_pointer_button_down_on();

        let bg_color = if is_down {
            colors::BLUE_700
        } else if is_hovered {
            colors::BLUE_600
        } else {
            colors::BLUE_500
        };

        // Glow sutil no hover
        let stroke = if is_hovered {
            Stroke::new(1.0_f32, colors::BLUE_300)
        } else {
            Stroke::NONE
        };

        ui.painter().rect(rect, radii::FULL, bg_color, stroke);
        ui.painter().galley(
            rect.center() - gallery.size() / 2.0,
            gallery,
            colors::TEXT_PRIMARY,
        );
    }

    if response.hovered() {
        ui.output_mut(|o| o.cursor_icon = egui::CursorIcon::PointingHand);
    }

    response
}

/// Botão secundário com alto contraste, mesma altura do primário (42px) e formato full rounded
pub fn secondary_ghost_button(
    ui: &mut egui::Ui,
    icon: Option<&str>,
    label: &str,
) -> egui::Response {
    let text = if let Some(ic) = icon {
        format!("{}  {}", ic, label)
    } else {
        label.to_string()
    };

    let font_id = typography::body_medium();
    let gallery = ui
        .painter()
        .layout_no_wrap(text, font_id, colors::SLATE_100);

    let size = egui::vec2(gallery.size().x + 36.0, 42.0);
    let (rect, response) = ui.allocate_exact_size(size, egui::Sense::click());

    if ui.is_rect_visible(rect) {
        let is_hovered = response.hovered();
        let is_down = response.is_pointer_button_down_on();

        // Fundo preto com feedback visual no hover e active
        let bg_color = if is_down {
            Color32::from_rgb(10, 10, 12)
        } else if is_hovered {
            Color32::from_rgb(26, 26, 30)
        } else {
            Color32::BLACK
        };

        // Sem borda no botão secundário
        let stroke = Stroke::NONE;

        let text_color = if is_hovered {
            colors::TEXT_PRIMARY
        } else {
            colors::SLATE_100
        };

        ui.painter().rect(rect, radii::FULL, bg_color, stroke);
        ui.painter()
            .galley(rect.center() - gallery.size() / 2.0, gallery, text_color);
    }

    if response.hovered() {
        ui.output_mut(|o| o.cursor_icon = egui::CursorIcon::PointingHand);
    }

    response
}

/// Botão de ação nos cards do grid (▶ Jogar)
#[allow(dead_code)]
pub fn card_action_button(
    ui: &mut egui::Ui,
    id: egui::Id,
    rect: egui::Rect,
    icon: &str,
    label: &str,
    is_primary: bool,
    hover_alpha: f32,
) -> egui::Response {
    let response = ui.interact(rect, id, egui::Sense::click());
    let is_btn_hovered = response.hovered();

    let alpha_u8 = (hover_alpha * 255.0).clamp(0.0, 255.0) as u8;

    let (bg_color, stroke, text_color) = if is_primary {
        if response.is_pointer_button_down_on() {
            (
                Color32::from_rgba_unmultiplied(29, 78, 216, alpha_u8), // Blue-700
                Stroke::NONE,
                Color32::from_rgba_unmultiplied(255, 255, 255, alpha_u8),
            )
        } else if is_btn_hovered {
            (
                Color32::from_rgba_unmultiplied(37, 99, 235, alpha_u8), // Blue-600
                Stroke::new(
                    1.0_f32,
                    Color32::from_rgba_unmultiplied(147, 197, 253, alpha_u8),
                ),
                Color32::from_rgba_unmultiplied(255, 255, 255, alpha_u8),
            )
        } else {
            (
                Color32::from_rgba_unmultiplied(59, 130, 246, alpha_u8), // Blue-500
                Stroke::NONE,
                Color32::from_rgba_unmultiplied(255, 255, 255, alpha_u8),
            )
        }
    } else {
        if response.is_pointer_button_down_on() {
            (
                Color32::from_rgba_unmultiplied(23, 37, 84, alpha_u8),
                Stroke::new(1.0_f32, Color32::from_rgba_unmultiplied(255, 255, 255, 90)),
                Color32::from_rgba_unmultiplied(255, 255, 255, alpha_u8),
            )
        } else if is_btn_hovered {
            (
                Color32::from_rgba_unmultiplied(30, 41, 59, alpha_u8),
                Stroke::new(
                    1.0_f32,
                    Color32::from_rgba_unmultiplied(96, 165, 250, alpha_u8),
                ),
                Color32::from_rgba_unmultiplied(255, 255, 255, alpha_u8),
            )
        } else {
            (
                Color32::from_rgba_unmultiplied(15, 23, 42, alpha_u8),
                Stroke::new(1.0_f32, Color32::from_rgba_unmultiplied(255, 255, 255, 35)),
                Color32::from_rgba_unmultiplied(226, 232, 240, alpha_u8),
            )
        }
    };

    // Sombra sutil projetada para reforçar a profundidade na descida do botão
    if hover_alpha > 0.05 {
        let shadow_rect = rect.translate(egui::vec2(0.0, 3.0)).expand(0.5);
        let shadow_alpha = (45.0 * hover_alpha).clamp(0.0, 255.0) as u8;
        ui.painter().rect_filled(
            shadow_rect,
            radii::FULL,
            Color32::from_black_alpha(shadow_alpha),
        );
    }

    ui.painter().rect(rect, radii::FULL, bg_color, stroke);

    let font_id = typography::body_medium();
    let full_text = format!("{}  {}", icon, label);
    ui.painter().text(
        rect.center(),
        egui::Align2::CENTER_CENTER,
        full_text,
        font_id,
        text_color,
    );

    if is_btn_hovered {
        ui.output_mut(|o| o.cursor_icon = egui::CursorIcon::PointingHand);
    }

    response
}

/// Botão de ícone no card (ex: alterar capa ao lado do botão de jogar)
pub fn card_icon_button(
    ui: &mut egui::Ui,
    id: egui::Id,
    rect: egui::Rect,
    icon: &str,
    tooltip: &str,
    hover_alpha: f32,
) -> egui::Response {
    let response = ui.interact(rect, id, egui::Sense::click());
    let is_btn_hovered = response.hovered();
    let alpha_u8 = (hover_alpha * 255.0).clamp(0.0, 255.0) as u8;

    let (bg_color, stroke, text_color) = if response.is_pointer_button_down_on() {
        (
            Color32::from_rgba_unmultiplied(15, 18, 26, alpha_u8),
            Stroke::NONE,
            Color32::from_rgba_unmultiplied(255, 255, 255, alpha_u8),
        )
    } else if is_btn_hovered {
        (
            Color32::from_rgba_unmultiplied(35, 45, 65, alpha_u8),
            Stroke::new(
                1.0_f32,
                Color32::from_rgba_unmultiplied(147, 197, 253, alpha_u8),
            ),
            Color32::from_rgba_unmultiplied(255, 255, 255, alpha_u8),
        )
    } else {
        (
            Color32::from_rgba_unmultiplied(18, 22, 32, (alpha_u8 as f32 * 0.9) as u8),
            Stroke::new(
                1.0_f32,
                Color32::from_rgba_unmultiplied(255, 255, 255, (25.0 * hover_alpha) as u8),
            ),
            Color32::from_rgba_unmultiplied(220, 226, 238, alpha_u8),
        )
    };

    if hover_alpha > 0.05 {
        let shadow_rect = rect.translate(egui::vec2(0.0, 2.0)).expand(0.5);
        let shadow_alpha = (40.0 * hover_alpha).clamp(0.0, 255.0) as u8;
        ui.painter().rect_filled(
            shadow_rect,
            radii::FULL,
            Color32::from_black_alpha(shadow_alpha),
        );
    }

    ui.painter().rect(rect, radii::FULL, bg_color, stroke);

    ui.painter().text(
        rect.center(),
        egui::Align2::CENTER_CENTER,
        icon,
        egui::FontId::proportional(17.0),
        text_color,
    );

    if is_btn_hovered {
        ui.output_mut(|o| o.cursor_icon = egui::CursorIcon::PointingHand);
    }

    response.on_hover_text(tooltip)
}

/// Badge normalizado padrão do Gank Design System (full rounded, vidro sutil e borda)
pub fn badge(ui: &mut egui::Ui, text: &str, icon: Option<&str>) {
    let font = typography::body_small();
    let display_text = if let Some(ic) = icon {
        format!("{}  {}", ic, text)
    } else {
        text.to_string()
    };
    let gallery = ui
        .painter()
        .layout_no_wrap(display_text, font, colors::SLATE_200);
    let size = egui::vec2(gallery.size().x + 20.0, 28.0);
    let (rect, _) = ui.allocate_exact_size(size, egui::Sense::hover());

    ui.painter().rect(
        rect,
        radii::FULL,
        colors::SURFACE_GLASS,
        Stroke::new(1.0_f32, colors::BORDER_SUBTLE),
    );
    ui.painter().galley(
        rect.center() - gallery.size() / 2.0,
        gallery,
        colors::SLATE_200,
    );
}

/// Badge de plataforma com estilo normalizado padrão
#[allow(dead_code)]
pub fn platform_badge(ui: &mut egui::Ui, platform: Platform) {
    badge(ui, platform.display_name(), None);
}

/// Tag de gênero com estilo normalizado padrão
#[allow(dead_code)]
pub fn genre_pill(ui: &mut egui::Ui, genre: &str) {
    badge(ui, genre, None);
}

/// Linha métrica sem wrapper box (utilizada no Showcase e nas páginas dos jogos)
/// Layout: Ícone branco ampliado (30px) perfeitamente centralizado na vertical à esquerda,
/// com Caption acima do Texto principal à direita.
pub fn stat_item(ui: &mut egui::Ui, icon: &str, label: &str, value: &str) {
    stat_item_alpha(ui, icon, label, value, 255);
}

/// Versão com suporte a controle de opacidade (alpha_u8) para animação de transição da página de detalhes
pub fn stat_item_alpha(ui: &mut egui::Ui, icon: &str, label: &str, value: &str, alpha_u8: u8) {
    ui.horizontal(|ui| {
        ui.spacing_mut().item_spacing = egui::vec2(14.0, 0.0);

        // Ícone ampliado (30px) perfeitamente alinhado ao centro vertical da linha
        let (rect, _) = ui.allocate_exact_size(egui::vec2(34.0, 42.0), egui::Sense::hover());
        ui.painter().text(
            rect.center(),
            egui::Align2::CENTER_CENTER,
            icon,
            egui::FontId::proportional(30.0),
            Color32::from_white_alpha(alpha_u8),
        );

        // Bloco vertical: Caption acima do Texto principal
        ui.vertical(|ui| {
            ui.spacing_mut().item_spacing = egui::vec2(0.0, 5.0);
            ui.label(
                egui::RichText::new(label)
                    .font(typography::caption())
                    .color(Color32::from_rgba_unmultiplied(
                        colors::SLATE_400.r(),
                        colors::SLATE_400.g(),
                        colors::SLATE_400.b(),
                        alpha_u8,
                    )),
            );
            ui.label(
                egui::RichText::new(value)
                    .font(typography::body_large())
                    .color(Color32::from_white_alpha(alpha_u8)),
            );
        });
    });
}

/// Card métrico legado de estatísticas da tela de detalhes
#[allow(dead_code)]
pub fn stat_card(
    ui: &mut egui::Ui,
    icon: &str,
    icon_color: Color32,
    label: &str,
    value: &str,
    width: f32,
    alpha_u8: u8,
) {
    let (rect, _) = ui.allocate_exact_size(egui::vec2(width, 62.0), egui::Sense::hover());
    if ui.is_rect_visible(rect) {
        let factor = alpha_u8 as f32 / 255.0;
        ui.painter().rect(
            rect,
            radii::MD,
            Color32::from_rgba_unmultiplied(15, 23, 42, (200.0 * factor) as u8),
            Stroke::new(
                1.0_f32,
                Color32::from_rgba_unmultiplied(255, 255, 255, (20.0 * factor) as u8),
            ),
        );

        // Ícone
        ui.painter().text(
            rect.left_center() + egui::vec2(14.0, 0.0),
            egui::Align2::LEFT_CENTER,
            icon,
            typography::title_large(),
            Color32::from_rgba_unmultiplied(
                icon_color.r(),
                icon_color.g(),
                icon_color.b(),
                alpha_u8,
            ),
        );

        // Label em caixa alta
        let tx = rect.left() + 44.0;
        ui.painter().text(
            egui::pos2(tx, rect.top() + 17.0),
            egui::Align2::LEFT_CENTER,
            label,
            typography::caption(),
            Color32::from_rgba_unmultiplied(
                colors::SLATE_400.r(),
                colors::SLATE_400.g(),
                colors::SLATE_400.b(),
                alpha_u8,
            ),
        );

        let disp_val = if value.len() > 18 {
            format!("{}...", &value[..16])
        } else {
            value.to_string()
        };
        ui.painter().text(
            egui::pos2(tx, rect.top() + 40.0),
            egui::Align2::LEFT_CENTER,
            disp_val,
            typography::body_medium(),
            Color32::from_white_alpha(alpha_u8),
        );
    }
}

/// Botão X para fechar modal ou detalhes
pub fn close_button(ui: &mut egui::Ui) -> egui::Response {
    let (rect, resp) = ui.allocate_exact_size(egui::vec2(32.0, 32.0), egui::Sense::click());
    let is_h = resp.hovered();

    let bg = if is_h {
        colors::SURFACE_GLASS_HOVER
    } else {
        colors::SURFACE_GLASS
    };

    let stroke = if is_h {
        Stroke::new(1.0_f32, colors::BLUE_400)
    } else {
        Stroke::new(1.0_f32, colors::BORDER_SUBTLE)
    };

    ui.painter().rect(rect, radii::FULL, bg, stroke);
    ui.painter().text(
        rect.center(),
        egui::Align2::CENTER_CENTER,
        egui_phosphor::regular::X,
        typography::title_medium(),
        if is_h {
            colors::TEXT_PRIMARY
        } else {
            colors::SLATE_300
        },
    );

    if is_h {
        ui.output_mut(|o| o.cursor_icon = egui::CursorIcon::PointingHand);
    }

    resp
}

/// Swatch card para exibir cada cor do Tailwind na galeria do Design System.
#[allow(dead_code)]
pub fn color_swatch(ui: &mut egui::Ui, name: &str, hex: &str, color: Color32, role: &str) {
    let width = 295.0;
    let height = 44.0;
    let (rect, _) = ui.allocate_exact_size(egui::vec2(width, height), egui::Sense::hover());

    // Fundo translúcido sutil e borda fina
    ui.painter().rect(
        rect,
        radii::MD,
        Color32::from_rgba_premultiplied(15, 23, 42, 140),
        Stroke::new(1.0_f32, colors::BORDER_SUBTLE),
    );

    // Amostra de cor à esquerda com borda suave
    let preview_size = 26.0;
    let preview_rect = egui::Rect::from_center_size(
        egui::pos2(rect.left() + 22.0, rect.center().y),
        egui::vec2(preview_size, preview_size),
    );
    ui.painter().rect(
        preview_rect,
        radii::SM,
        color,
        Stroke::new(1.0_f32, Color32::from_rgba_premultiplied(255, 255, 255, 30)),
    );

    // 1. Título (Nome da Cor)
    let text_x = rect.left() + 44.0;
    ui.painter().text(
        egui::pos2(text_x, rect.center().y),
        egui::Align2::LEFT_CENTER,
        name,
        typography::body_medium(),
        colors::TEXT_PRIMARY,
    );

    // 2. Descrição / Papel lado a lado
    let role_x = text_x + 78.0;
    ui.painter().text(
        egui::pos2(role_x, rect.center().y),
        egui::Align2::LEFT_CENTER,
        role,
        typography::body_small(),
        colors::TEXT_MUTED,
    );

    // 3. Hexadecimal alinhado à direita na mesma linha
    let hex_x = rect.right() - 14.0;
    ui.painter().text(
        egui::pos2(hex_x, rect.center().y),
        egui::Align2::RIGHT_CENTER,
        hex,
        typography::monospace(),
        colors::BLUE_300,
    );
}

/// Campo de busca integrado com ícone de lupa, hint e botão de limpar
#[allow(dead_code)]
pub fn search_input(ui: &mut egui::Ui, query: &mut String, width: f32) -> egui::Response {
    let desired_size = egui::vec2(width, 34.0);
    let (rect, response) = ui.allocate_exact_size(desired_size, egui::Sense::hover());

    if ui.is_rect_visible(rect) {
        let is_hovered = ui.rect_contains_pointer(rect);
        let edit_id = response.id.with("search_text_edit");
        let has_focus = ui.memory(|m| m.has_focus(edit_id));

        let bg_color = if has_focus {
            Color32::from_rgba_unmultiplied(24, 30, 44, 230)
        } else if is_hovered {
            colors::SURFACE_GLASS_HOVER
        } else {
            colors::SURFACE_GLASS
        };

        let stroke = if has_focus {
            Stroke::new(1.0_f32, colors::BLUE_500)
        } else if is_hovered {
            Stroke::new(1.0_f32, colors::BORDER_SUBTLE)
        } else {
            Stroke::new(1.0_f32, Color32::from_white_alpha(18))
        };

        ui.painter().rect(rect, radii::FULL, bg_color, stroke);

        // Ícone de lupa à esquerda
        let icon_x = rect.min.x + 16.0;
        ui.painter().text(
            egui::pos2(icon_x, rect.center().y),
            egui::Align2::CENTER_CENTER,
            egui_phosphor::regular::MAGNIFYING_GLASS,
            egui::FontId::proportional(15.0),
            if has_focus {
                colors::BLUE_400
            } else {
                colors::TEXT_MUTED
            },
        );

        // Botão de limpar à direita (X) quando há texto
        let clear_w = if !query.is_empty() { 24.0 } else { 0.0 };
        if !query.is_empty() {
            let clear_rect = egui::Rect::from_center_size(
                egui::pos2(rect.max.x - 16.0, rect.center().y),
                egui::vec2(20.0, 20.0),
            );
            let clear_resp = ui.interact(
                clear_rect,
                response.id.with("clear_search_btn"),
                egui::Sense::click(),
            );
            if clear_resp.clicked() {
                query.clear();
            }
            let clear_color = if clear_resp.hovered() {
                Color32::WHITE
            } else {
                colors::TEXT_MUTED
            };
            ui.painter().text(
                clear_rect.center(),
                egui::Align2::CENTER_CENTER,
                egui_phosphor::regular::X,
                egui::FontId::proportional(13.0),
                clear_color,
            );
            if clear_resp.hovered() {
                ui.output_mut(|o| o.cursor_icon = egui::CursorIcon::PointingHand);
            }
        }

        // Input de texto
        let text_rect = egui::Rect::from_min_max(
            egui::pos2(rect.min.x + 28.0, rect.min.y + 2.0),
            egui::pos2(rect.max.x - 12.0 - clear_w, rect.max.y - 2.0),
        );

        let mut child_ui = ui.child_ui(text_rect, egui::Layout::left_to_right(egui::Align::Center));
        child_ui.style_mut().visuals.extreme_bg_color = Color32::TRANSPARENT;
        child_ui.style_mut().visuals.widgets.hovered.bg_stroke = Stroke::NONE;
        child_ui.style_mut().visuals.widgets.active.bg_stroke = Stroke::NONE;
        child_ui.style_mut().visuals.widgets.inactive.bg_stroke = Stroke::NONE;
        child_ui
            .style_mut()
            .visuals
            .widgets
            .noninteractive
            .bg_stroke = Stroke::NONE;

        let edit = egui::TextEdit::singleline(query)
            .id(edit_id)
            .hint_text(
                egui::RichText::new("Buscar... (Ctrl+F)")
                    .color(colors::TEXT_MUTED)
                    .size(13.0),
            )
            .text_color(colors::TEXT_PRIMARY)
            .font(typography::body_medium())
            .frame(false)
            .margin(egui::vec2(0.0, 0.0));

        child_ui.add(edit);
    }

    response
}
