//!$ notes.rs

#[allow(unused)]
fn enable_fuzzy_search(&self) {
    self.fuzzy_search_open.store(true, Ordering::Relaxed);
}

#[allow(unused)]
fn disable_fuzzy_search(&self) {
    self.fuzzy_search_open.store(false, Ordering::Relaxed);
}

#[allow(unused)]
fn is_fuzzy_search_open(&self) -> bool {
    self.fuzzy_search_open.load(Ordering::Relaxed)
}
