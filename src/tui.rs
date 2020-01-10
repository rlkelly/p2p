use std::sync::Arc;
use tokio::sync::{Mutex, MutexGuard};
use cursive::Cursive;
use cursive::traits::*;
use cursive::align::HAlign;
use cursive::views::{Dialog, SelectView};

pub use crate::models::{ArtistData, Service};

pub fn run_tui(state: Arc<Mutex<Service>>) {
    let mut siv = Cursive::default();
    siv.add_global_callback('q', |s| s.quit());
    siv.add_layer(Dialog::text("Select Option")
        .title("Main Menu")
        .button("Next", move |s| main_menu(s, Arc::clone(&state))));
    siv.run();
}

fn main_menu(s: &mut Cursive, state: Arc<Mutex<Service>>) {
    // s.pop_layer();
    let mut select = SelectView::new()
        .h_align(HAlign::Center)
        .autojump();
    let content = "peers\nlibrary\n";
    select.add_all_str(content.lines());
    let collection = futures::executor::block_on(get_state(state));
    select.set_on_submit(move |s, m| select_submenu(s, m, &collection));
    s.add_layer(
        Dialog::around(select.scrollable().fixed_size((20, 10)))
            .title("Main Menu"),
    );
}

fn select_submenu(s: &mut Cursive, m: &str, collection: &Vec<ArtistData>) {
    s.pop_layer();
    let mut select = SelectView::new()
        .h_align(HAlign::Center)
        .autojump();
    let mut content = String::new();
    for artist in collection {
        content.push_str(&artist.artist);
        content.push_str("\n");
    }
    select.add_all_str(content.lines());
    s.add_layer(
        Dialog::around(select.scrollable().fixed_size((20, 10)))
            .title(m),
    );
}

async fn get_state(state: Arc<Mutex<Service>>) -> Vec<ArtistData> {
    let s = state.lock().await;
    s.get_collection(false, None, None)
}
