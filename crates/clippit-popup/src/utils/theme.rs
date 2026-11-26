use clippit_core::Config;
use libadwaita as adw;

/// Applies the theme configuration from the config
///
/// # Arguments
/// * `config` - The configuration containing theme settings
pub fn apply_theme(_config: &Config) {
    // O popup tem visual próprio escuro (paleta em load_custom_css), então força
    // o esquema escuro do libadwaita para os widgets internos combinarem
    adw::StyleManager::default().set_color_scheme(adw::ColorScheme::ForceDark);
}

/// Loads custom CSS for the application
///
/// Includes rounded corners for thumbnails and preview images
pub fn load_custom_css() {
    let css_provider = gtk::CssProvider::new();
    css_provider.load_from_data(
        "image.thumbnail-rounded {
            border-radius: 12px;
        }
        image.preview-rounded {
            border-radius: 16px;
        }
        
        /* Skeleton loader styles - matches Adwaita design */
        .skeleton-thumb {
            background: linear-gradient(90deg, 
                alpha(@window_fg_color, 0.08) 0%, 
                alpha(@window_fg_color, 0.12) 50%, 
                alpha(@window_fg_color, 0.08) 100%);
            background-size: 200% 100%;
            border-radius: 12px;
        }
        
        .skeleton-text {
            background: linear-gradient(90deg, 
                alpha(@window_fg_color, 0.08) 0%, 
                alpha(@window_fg_color, 0.12) 50%, 
                alpha(@window_fg_color, 0.08) 100%);
            background-size: 200% 100%;
            border-radius: 6px;
        }
        
        @keyframes skeleton-pulse {
            0% {
                background-position: 200% 0;
            }
            100% {
                background-position: -200% 0;
            }
        }
        
        .skeleton-pulse {
            animation: skeleton-pulse 1.8s ease-in-out infinite;
        }
        
        /* Popover de sugestões de autocompletar */
        .suggestions-popover {
            padding: 4px;
            background-color: @window_bg_color;
            border-radius: 8px;
            box-shadow: 0 2px 8px rgba(0, 0, 0, 0.15);
        }
        
        .suggestions-list {
            background-color: transparent;
        }
        
        .suggestion-row {
            padding: 8px 12px;
            border-radius: 6px;
            transition: background-color 0.15s ease;
        }
        
        .suggestion-row:hover {
            background-color: alpha(@accent_color, 0.1);
        }
        
        .suggestion-row:selected {
            background-color: @accent_color;
            color: @accent_fg_color;
        }
        
        .suggestion-word {
            font-weight: 500;
            font-size: 14px;
        }
        
        .suggestion-history-icon {
            color: @success_color;
            opacity: 0.9;
        }
        
        .suggestion-row:selected .suggestion-history-icon {
            color: @accent_fg_color;
            opacity: 1;
        }
        
        /* ===== Visual moderno escuro ===== */
        @define-color clip_bg #1e1e1e;
        @define-color clip_card #262626;
        @define-color clip_card_hover #2c2c2c;
        @define-color clip_border #333333;
        @define-color clip_border_strong #424242;
        @define-color clip_text #f2f2f2;
        @define-color clip_muted #8f8f8f;
        @define-color clip_accent #4c9dff;
        @define-color clip_accent_bg #1c2f45;
        @define-color clip_danger #ff6b6b;
        @define-color clip_danger_bg #3a1f22;

        window.clippit-popup,
        window.clippit-popup .clip-root {
            background-color: @clip_bg;
            color: @clip_text;
        }
        window.clippit-popup {
            border-radius: 18px;
            border: 1px solid @clip_border;
        }

        entry.clip-search {
            min-height: 44px;
            padding: 0 14px;
            border-radius: 13px;
            background-color: @clip_card;
            border: 1px solid @clip_border;
            color: @clip_text;
            box-shadow: none;
            outline: none;
        }
        entry.clip-search:focus-within {
            border-color: @clip_accent;
            box-shadow: 0 0 0 3px alpha(@clip_accent, 0.15);
        }
        entry.clip-search image { color: @clip_muted; }

        .clip-scroll,
        .clip-scroll > viewport,
        list.clip-list {
            background-color: transparent;
        }
        list.clip-list > row {
            background-color: @clip_card;
            border: 1px solid @clip_border;
            border-radius: 14px;
            margin-bottom: 8px;
            padding: 4px 4px;
            transition: background-color 120ms ease, border-color 120ms ease;
            outline: none;
        }
        list.clip-list > row:hover {
            background-color: @clip_card_hover;
            border-color: @clip_border_strong;
        }
        list.clip-list > row:selected {
            background-color: @clip_accent_bg;
            border-color: alpha(@clip_accent, 0.7);
        }
        list.clip-list > row .title {
            color: @clip_text;
            font-size: 14px;
            font-weight: 500;
        }
        list.clip-list > row .subtitle {
            color: @clip_muted;
            font-size: 12px;
        }

        button.clip-action {
            min-width: 34px;
            min-height: 34px;
            padding: 0;
            border-radius: 10px;
            background: alpha(white, 0.05);
            color: #b5b5b5;
            border: none;
            box-shadow: none;
            margin-left: 6px;
        }
        button.clip-copy:hover,
        button.clip-expand:hover {
            background-color: @clip_accent_bg;
            color: @clip_accent;
        }
        button.clip-delete:hover {
            background-color: @clip_danger_bg;
            color: @clip_danger;
        }
        list.clip-list > row:selected button.clip-copy {
            background-color: alpha(@clip_accent, 0.18);
            color: @clip_accent;
        }

        image.thumbnail-rounded {
            border-radius: 10px;
        }

        .clip-footer {
            padding: 12px 16px;
            border-radius: 14px;
            background-color: @clip_card;
            border: 1px solid @clip_border;
        }
        .clip-key {
            font-size: 11px;
            font-weight: 700;
            color: @clip_text;
            padding: 2px 8px;
            border-radius: 6px;
            background-color: alpha(white, 0.08);
            border: 1px solid @clip_border_strong;
        }
        .clip-hint {
            font-size: 12px;
            color: @clip_muted;
        }

        .clip-lightbox {
            background-color: #0e0e0e;
            border-radius: 18px;
            padding: 14px 16px 16px 16px;
        }
        .clip-lightbox-info {
            color: @clip_muted;
            font-size: 13px;
            font-weight: 600;
        }
        .clip-zoom-label {
            color: @clip_text;
            font-size: 12px;
            font-weight: 700;
            font-feature-settings: 'tnum';
        }
        .clip-lightbox-view,
        .clip-lightbox-view > viewport {
            background-color: #0e0e0e;
            border-radius: 12px;
        }

        .suggestions-popover > contents {
            background-color: @clip_card;
            border: 1px solid @clip_border;
            border-radius: 12px;
        }

        /* Garante que botões de ação sejam visíveis em qualquer tema */
        button.flat.circular {
            opacity: 0.7;
            transition: opacity 0.2s ease;
        }
        
        button.flat.circular:hover {
            opacity: 1;
        }",
    );

    if let Some(display) = gtk::gdk::Display::default() {
        gtk::style_context_add_provider_for_display(
            &display,
            &css_provider,
            gtk::STYLE_PROVIDER_PRIORITY_APPLICATION,
        );
    }
}
