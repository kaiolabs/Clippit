use gtk::prelude::*;
use libadwaita as adw;
use std::rc::Rc;
use std::cell::RefCell;
use crate::utils::create_thumbnail;

/// Adds a hover preview popover to an image row
/// Shows a larger 512px preview when hovering over the thumbnail
pub fn add_image_hover_preview(row: &adw::ActionRow, data: &[u8]) {
    let data_clone = data.to_vec();
    let row_clone = row.clone();
    
    let hover = gtk::EventControllerMotion::new();
    let popover: Rc<RefCell<Option<gtk::Popover>>> = Rc::new(RefCell::new(None));
    let popover_clone = popover.clone();
    
    hover.connect_enter(move |_, _, _| {
        // Create preview popover if not exists
        if popover_clone.borrow().is_none() {
            if let Ok(preview_pixbuf) = create_thumbnail(&data_clone, 512) {
                let preview_image = gtk::Image::from_pixbuf(Some(&preview_pixbuf));
                preview_image.set_margin_top(8);
                preview_image.set_margin_bottom(8);
                preview_image.set_margin_start(8);
                preview_image.set_margin_end(8);
                preview_image.add_css_class("preview-rounded");
                
                let new_popover = gtk::Popover::new();
                new_popover.set_child(Some(&preview_image));
                new_popover.set_parent(&row_clone);
                new_popover.set_position(gtk::PositionType::Right);
                new_popover.set_autohide(false);
                new_popover.set_can_focus(false);
                
                *popover_clone.borrow_mut() = Some(new_popover);
            }
        }
        
        // Show popover
        if let Some(ref pop) = *popover_clone.borrow() {
            pop.popup();
        }
    });
    
    let popover_clone2 = popover.clone();
    hover.connect_leave(move |_| {
        // Hide popover
        if let Some(ref pop) = *popover_clone2.borrow() {
            pop.popdown();
        }
    });
    
    row.add_controller(hover);
}
