mod error_handling;
mod spotify;

use error_handling::{Error, Result};
use rand::seq::SliceRandom;
use rspotify::{client::Spotify as SpotifyClient, model::track::SavedTrack};
use spotify::{get_spotify_client, play};
use std::{env, error, fs, future::Future, path::PathBuf, pin::Pin};

const TRACKS_LIMIT: usize = 666;

#[tokio::main]
async fn main() -> std::result::Result<(), Box<dyn error::Error>> {
    let client = get_spotify_client(get_cache_path()?).await?;
    let tracks_uris = get_tracks_uris(client.clone()).await;
    play(client, tracks_uris).await?;
    Ok(())
}

fn get_cache_path() -> Result<PathBuf> {
    match env::var_os("DIONI_CACHE")
        .and_then(|s| Some(PathBuf::from(s)))
        .or(env::var_os("XDG_CACHE_HOME").and_then(|s| {
            let mut p = PathBuf::from(s);
            p.push("dioni");
            Some(p)
        }))
        .or(dirs_next::cache_dir())
    {
        Some(mut p) => {
            fs::create_dir_all(&p)?;
            p.push(".spotify_token");
            Ok(p)
        }
        None => Err(Error::UnkownCachePath),
    }
}

async fn get_tracks_uris(client: SpotifyClient) -> Vec<String> {
    let mut tracks = Vec::<SavedTrack>::new();
    get_all_saved_tracks(client, &mut tracks, 0).await;
    let mut rng = rand::thread_rng();
    tracks.shuffle(&mut rng);
    tracks
        .iter()
        .take(TRACKS_LIMIT)
        .map(|x| x.track.uri.clone())
        .collect::<Vec<String>>()
}

fn get_all_saved_tracks<'a>(
    client: SpotifyClient,
    tracks: &'a mut Vec<SavedTrack>,
    offset: u32,
) -> Pin<Box<dyn Future<Output = ()> + 'a>> {
    Box::pin(async move {
        match client
            .current_user_saved_tracks(Some(50), Some(offset))
            .await
        {
            Ok(mut response) => {
                if response.items.len() == 0 {
                    return;
                }
                tracks.append(&mut response.items);
                get_all_saved_tracks(client, tracks, offset + 50).await;
            }
            Err(_) => return,
        }
    })
}
