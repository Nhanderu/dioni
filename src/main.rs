mod error_handling;
mod spotify;

use error_handling::{Error, Result};
use rand::seq::SliceRandom;
use rspotify::{client::Spotify as SpotifyClient, model::track::SavedTrack};
use spotify::{get_spotify_client, play};
use std::{
    env, fs,
    future::Future,
    io::{stdin, stdout, Write},
    path::PathBuf,
    pin::Pin,
    process::exit,
};

const TRACKS_LIMIT: usize = 666;

#[tokio::main]
async fn main() {
    if let Err(e) = run().await {
        eprintln!("Error: {}.", e);
        exit(1);
    }
}

async fn run() -> Result<()> {
    let client = get_spotify_client(get_cache_path()?).await?;
    let (tracks_uris, queue_uris) = get_tracks_uris(client.clone()).await;
    play(client, tracks_uris, queue_uris).await?;
    println!("Enjoy your music. :)");
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

async fn get_tracks_uris(client: SpotifyClient) -> (Vec<String>, Vec<String>) {
    let mut tracks = Vec::<SavedTrack>::new();
    get_all_saved_tracks(client, &mut tracks, 0).await;
    clear_stdout_line();
    println!("{} saved tracks found in total.", tracks.len());

    tracks.shuffle(&mut rand::thread_rng());
    let iter = tracks.iter().map(|x| x.track.uri.clone());
    let tracks_uris = iter.clone().take(TRACKS_LIMIT).collect();

    if tracks.len() <= TRACKS_LIMIT {
        return (tracks_uris, Vec::new());
    }

    println!("The total saved tracks is above our limit.");
    println!("Do you want to add the rest in the queue? If you choose no, they'll be ignored.");
    println!("(everything besides 'yes' will be considered as 'no')");
    let mut input = String::new();
    
    if stdin().read_line(&mut input).is_err() {
        println!("Error reading your option; 'no' was assumed.");
        return (tracks_uris, Vec::new());
    }
    if input.trim() != "yes" {
        return (tracks_uris, Vec::new());
    }
    (tracks_uris, iter.skip(TRACKS_LIMIT).collect())
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
                clear_stdout_line();
                print!("{} saved tracks found.", tracks.len());
                get_all_saved_tracks(client, tracks, offset + 50).await;
            }
            Err(_) => return,
        }
    })
}

fn clear_stdout_line() {
    print!("\r");
    let _ = stdout().flush();
}
