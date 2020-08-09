mod args_parsing;
mod error_handling;
mod spotify;
mod tracks;
mod utils;

use args_parsing::ARGS;
use error_handling::{Error, Result};
use spotify::{get_spotify_client, play};
use std::{env, fs, path::PathBuf, process::exit};
use tracks::get_shuffled_tracks_uris;

#[tokio::main]
async fn main() {
    if let Err(e) = run().await {
        eprintln!("Error: {}.", e);
        exit(1);
    }
}

async fn run() -> Result<()> {
    let cache_path = get_cache_path()?;
    let client = get_spotify_client(cache_path.clone()).await?;
    let (tracks_uris, queue_uris) = get_shuffled_tracks_uris(cache_path, client.clone()).await;
    play(client, tracks_uris, queue_uris).await?;
    cond_println!("Enjoy your music. :)");
    Ok(())
}

pub fn get_cache_path() -> Result<PathBuf> {
    match env::var_os("DIONI_CACHE")
        .and_then(|s| Some(PathBuf::from(s)))
        .or(env::var_os("XDG_CACHE_HOME").and_then(|s| {
            let mut p = PathBuf::from(s);
            p.push("dioni");
            Some(p)
        }))
        .or(dirs_next::cache_dir().and_then(|mut p| {
            p.push("dioni");
            Some(p)
        })) {
        Some(p) => {
            fs::create_dir_all(&p)?;
            Ok(p)
        }
        None => Err(Error::UnkownCachePath),
    }
}
