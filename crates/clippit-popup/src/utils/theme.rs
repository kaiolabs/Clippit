use libadwaita as adw;
use clippit_core::Config;

/// Applies the theme configuration from the config
/// 
/// # Arguments
/// * `config` - The configuration containing theme settings
pub fn apply_theme(config: &Config) {
    let style_manager = adw::StyleManager::default();
    
    match config.ui.theme.as_str() {
        "dark" => style_manager.set_color_scheme(adw::ColorScheme::ForceDark),
        "light" => style_manager.set_color_scheme(adw::ColorScheme::ForceLight),
        "system" | _ => style_manager.set_color_scheme(adw::ColorScheme::Default),
    }
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
        }"
    );
    
    if let Some(display) = gtk::gdk::Display::default() {
        gtk::style_context_add_provider_for_display(
            &display,
            &css_provider,
            gtk::STYLE_PROVIDER_PRIORITY_APPLICATION,
        );
    }
}
