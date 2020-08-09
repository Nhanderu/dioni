use super::{args_parsing::ARGS, cond_print, cond_println, utils::clear_stdout_line};
use rand::seq::SliceRandom;
use rspotify::{client::Spotify as SpotifyClient, model::track::SavedTrack};
use std::{error, future::Future, io::stdin, path::PathBuf, pin::Pin, fs::File,io::{BufReader, BufWriter}};
use serde_json;

const TRACKS_LIMIT: usize = 666;

pub async fn get_shuffled_tracks_uris(
    cache_path: PathBuf,
    client: SpotifyClient,
) -> (Vec<String>, Vec<String>) {

    let tracks_uris = get_tracks_uris(cache_path, client).await;
    cond_println!("{} saved tracks found in total.", tracks_uris.len());

    if tracks_uris.len() <= TRACKS_LIMIT {
        return (tracks_uris, Vec::new());
    }


    let iter = tracks_uris.iter().map(|x| x.to_string());
    let limited = iter.clone().take(TRACKS_LIMIT).collect();

    if ARGS.ignore_excess {
        cond_println!(
            "The total saved tracks is above our limit and the excess is going to be ignored."
        );
        return (limited, Vec::new());
    }

    if ARGS.add_excess_to_queue {
        cond_println!(
            "The total saved tracks is above our limit and the excess is going to be added to the queue."
        );
    } else {
        cond_println!("The total saved tracks is above our limit.");
        cond_println!(
            "Do you want to add the rest in the queue? If you choose no, they'll be ignored."
        );
        cond_println!("(everything besides 'yes' will be considered as 'no')");
        let mut input = String::new();
        if stdin().read_line(&mut input).is_err() {
            cond_println!("Error reading your option; 'no' was assumed.");
            return (limited, Vec::new());
        }
        if input.trim() != "yes" {
            return (limited, Vec::new());
        }
    }

    (limited, iter.skip(TRACKS_LIMIT).collect())
}

async fn get_tracks_uris(mut cache_path: PathBuf, client: SpotifyClient) -> Vec<String> {
    cache_path.push(".saved-tracks");
    if !ARGS.force_fetching {
        match get_cached_tracks_uris(cache_path.clone()).await {
            Ok(tracks) => {
                cond_println!("Tracks retrieved from cache.");
                return tracks;
            },
            Err(_) => {}
        };
    }

    let mut tracks = Vec::<SavedTrack>::new();
    fetch_saved_tracks(client, &mut tracks, 0).await;
    clear_stdout_line();

    tracks.shuffle(&mut rand::thread_rng());
    let tracks_uris: Vec<String> = tracks.iter().map(|x| x.track.uri.clone()).collect();
    let _ = cache_tracks_uris(cache_path, tracks_uris.clone());
    tracks_uris
}

async fn get_cached_tracks_uris(cache_path: PathBuf) -> Result<Vec<String>, Box<dyn error::Error>> {
    let file = File::open(cache_path)?;
    Ok(serde_json::from_reader(BufReader::new(file))?)
}

fn cache_tracks_uris(cache_path: PathBuf, tracks_uris: Vec<String>) -> Result<(), Box<dyn error::Error>> {
    let mut file = File::create(cache_path)?;
    serde_json::to_writer(BufWriter::new(&mut file), &tracks_uris)?;
    Ok(())
}

fn fetch_saved_tracks<'a>(
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
                cond_print!("{} saved tracks found.", tracks.len());
                fetch_saved_tracks(client, tracks, offset + 50).await;
            }
            Err(_) => return,
        }
    })
}
