use rand::seq::SliceRandom;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uqbar_process_lib::{
    await_message, call_init, http, print_to_terminal, Address, Message,
};

// TODO: replace with VFS reads inside init()
const SONGS: &str = include_str!("../../pkg/songs.json");
const LYRICS: &str = include_str!("../../pkg/lyrics.json");

#[derive(Serialize, Deserialize)]
struct Album {
    album: String,
    year: u16,
    //          title,  lyrics (grouped into 'quotes')
    songs: Vec<(String, Vec<Vec<String>>)>,
}

#[derive(Serialize, Deserialize)]
struct Lyric {
    album: String,
    year: u16,
    song: String,
    lyric: Vec<String>,
}

wit_bindgen::generate!({
    path: "wit",
    world: "process",
    exports: {
        world: Component,
    },
});

call_init!(init);

fn init(_our: Address) {
    print_to_terminal(0, "SteelyDanAPI: online");

    // parse files
    let songs: Vec<String> = serde_json::from_str(SONGS).expect("failed to parse songs");
    let lyrics: Vec<Album> = serde_json::from_str(LYRICS).expect("failed to parse lyrics");

    // bind endpoints for public access
    http::bind_http_path("/song", false, false).expect("failed to bind /song");
    http::bind_http_path("/lyric", false, false).expect("failed to bind /lyric");

    loop {
        let Ok(Message::Request { ref ipc, .. }) = await_message() else {
            continue
        };

        let Ok(http::HttpServerRequest::Http(incoming)) = serde_json::from_slice::<http::HttpServerRequest>(ipc) else {
            continue
        };

        if incoming.method != "GET" {
            continue;
        };

        let seed = &mut rand::thread_rng();

        match incoming.path().unwrap_or_default().as_str() {
            "song" => {
                // select random song from list
                http::send_response(
                    http::StatusCode::OK,
                    Some(HashMap::from([(
                        "Content-Type".to_string(),
                        "text/plain".to_string(),
                    )])),
                    songs.choose(seed).unwrap().as_bytes().to_vec(),
                ).unwrap();
            }
            "lyric" => {
                // select random album, random song, then random lyric snippet
                let album = lyrics.choose(seed).unwrap();
                let (song_title, lyrics) = album.songs.choose(seed).unwrap();
                let lyric = lyrics.choose(seed).unwrap();
                http::send_response(
                    http::StatusCode::OK,
                    Some(HashMap::from([(
                        "Content-Type".to_string(),
                        "application/json".to_string(),
                    )])),
                    serde_json::to_vec(&Lyric {
                        album: album.album.to_string(),
                        year: album.year,
                        song: song_title.to_string(),
                        lyric: lyric.to_vec(),
                    })
                    .unwrap(),
                ).unwrap();
            }
            _ => {
                http::send_response(
                    http::StatusCode::NOT_FOUND,
                    None,
                    "404 Not Found".as_bytes().to_vec(),
                ).unwrap();
            }
        }
    }
}
