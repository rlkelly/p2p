use std::sync::Arc;

use cursive::Cursive;
use cursive::traits::*;
use cursive::align::HAlign;
use cursive::views::{BoxView, Dialog, SelectView};

use tokio::sync::Mutex;

pub use crate::models::{ArtistData, Peer, Service};

pub fn run_tui(state: Arc<Mutex<Service>>) {
    let mut siv = Cursive::default();
    siv.add_global_callback('q', |s| s.quit());
    siv.add_layer(Dialog::text("Select Option")
        .title("Welcome")
        .button("Next", move |s| main_menu(s, Arc::clone(&state))));
    siv.run();
}

fn main_menu(s: &mut Cursive, state: Arc<Mutex<Service>>) {
    s.pop_layer();
    let mut select = SelectView::<String>::new()
        .h_align(HAlign::Center)
        .autojump();
    let content = "peers\nlibrary\n";
    select.add_all_str(content.lines());
    select.set_on_submit(move |s, m| select_submenu(s, m, Arc::clone(&state)));
    let box_select = BoxView::with_fixed_size((20, 10), select);
    s.add_layer(Dialog::around(box_select.scrollable())
        .h_align(HAlign::Center)
        .title("Options")
    );
}

fn select_submenu(s: &mut Cursive, m: &str, state: Arc<Mutex<Service>>) {
    if m == "library" {
        let collection = futures::executor::block_on(get_collection(state));
        show_collection(s, collection);
    } else {
        let peers = futures::executor::block_on(get_peers(state));
        show_peers(s, peers);
    }
}

fn show_peers(s: &mut Cursive, peers: Vec<Peer>) {
    let mut select = SelectView::new()
        .h_align(HAlign::Center)
        .autojump();
    let mut content = String::new();
    for peer in &peers {
        content.push_str(peer.name.as_ref().unwrap_or(&"unk".to_string()));
        content.push_str("\n");
    }
    select.add_all_str(content.lines());
    // select.set_on_submit(move |s, m| show_albums(s, m, collection.clone()));
    s.add_layer(
        Dialog::around(select.scrollable().fixed_size((20, 10)))
            .title("Peers")
            .button("Back", |s| {s.pop_layer();})
    );
}

fn show_collection(s: &mut Cursive, collection: Vec<ArtistData>) {
    let mut select = SelectView::new()
        .h_align(HAlign::Center)
        .autojump();
    let mut content = String::new();
    for artist in &collection {
        content.push_str(&artist.artist);
        content.push_str("\n");
    }
    select.add_all_str(content.lines());
    select.set_on_submit(move |s, m| show_albums(s, m, collection.clone()));
    let box_select = BoxView::with_fixed_size((20, 10), select);
    s.add_layer(
        Dialog::around(box_select.scrollable())
            .title("Library")
            .button("Back", |s| {s.pop_layer();})
    );
}

fn show_albums(s: &mut Cursive, artist_name: &str, collection: Vec<ArtistData>) {
    let mut select = SelectView::new()
        .h_align(HAlign::Center)
        .autojump();
    // TODO: save album
    let mut content = String::new();
    for artist in &collection {
        if artist.artist == artist_name {
            for album in artist.albums.as_ref().unwrap() {
                content.push_str(&album.album_title);
                content.push_str("\n");
            }
            break;
        }
    }
    select.add_all_str(content.lines());
    let name = String::from(artist_name);
    select.set_on_submit(move |s, m| show_tracks(s, &name, m, &collection));
    let box_select = BoxView::with_fixed_size((30, 10), select);
    s.add_layer(
        Dialog::around(box_select.scrollable())
            .title(format!("__{}__", artist_name))
            .button("Back", |s| {s.pop_layer();}),
    );
}

fn show_tracks(s: &mut Cursive, artist_name: &str, album_title: &str, collection: &Vec<ArtistData>) {
    let mut select = SelectView::new()
        .h_align(HAlign::Left)
        .autojump();
    let mut content = String::new();
    'outer: for artist in collection {
        if artist.artist == artist_name {
            for album in artist.albums.as_ref().unwrap() {
                if album.album_title == album_title {
                    if let Some(tracks) = &album.tracks {
                        let mut track_count = 1;
                        for track in tracks {
                            content.push_str(&format!("{}. {}", &track_count, &track.title));
                            content.push_str("\n");
                            track_count += 1;
                        }
                        break 'outer;
                    }
                }
            }
        }
    }
    select.add_all_str(content.lines());
    let box_select = BoxView::with_fixed_size((30, 10), select.scrollable());
    s.add_layer(
        Dialog::around(box_select)
            .title(album_title)
            .button("Back", |s| {s.pop_layer();}),
    );
}

async fn get_collection(state: Arc<Mutex<Service>>) -> Vec<ArtistData> {
    let s = state.lock().await;
    s.get_collection(true, None, None)
}

async fn get_peers(state: Arc<Mutex<Service>>) -> Vec<Peer> {
    let s = state.lock().await;
    s.get_peers()
}
