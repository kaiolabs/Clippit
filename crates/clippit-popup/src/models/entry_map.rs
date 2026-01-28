use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

/// Manages the mapping between row indices and entry IDs
pub type EntryMap = Rc<RefCell<HashMap<i32, i64>>>;

/// Manages the mapping between row indices and search content
pub type SearchContentMap = Rc<RefCell<HashMap<i32, String>>>;

/// Creates a new EntryMap
pub fn new_entry_map() -> EntryMap {
    Rc::new(RefCell::new(HashMap::new()))
}

/// Creates a new SearchContentMap
pub fn new_search_content_map() -> SearchContentMap {
    Rc::new(RefCell::new(HashMap::new()))
}
