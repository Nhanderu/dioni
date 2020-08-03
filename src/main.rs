mod error;

extern crate dirs_next;
extern crate rspotify;

use std::env;
use std::error::Error;
use std::fs;
use std::io::prelude::*;
use std::net::TcpListener;
use std::path::PathBuf;

use error::{CachePathError, CachePathErrorType};

use rspotify::client::Spotify;
use rspotify::oauth2::{SpotifyClientCredentials, SpotifyOAuth, TokenInfo};
use rspotify::util::{generate_random_string, process_token};

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let cache_path = get_cache_path()?;
    let mut auth = SpotifyOAuth::default()
        .redirect_uri("http://localhost:29797/")
        .cache_path(cache_path)
        .build();
    match get_token(&mut auth).await {
        Some(token_info) => {
            let creds = SpotifyClientCredentials::default()
                .token_info(token_info)
                .build();
            let spotify = Spotify::default().client_credentials_manager(creds).build();
            let me = spotify.me().await;
            println!("{:?}", me.unwrap());
        }
        None => println!("auth failed"),
    };
    Ok(())
}

fn get_cache_path() -> Result<PathBuf, CachePathError> {
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
            p.push(".spotify_token_cache.json");
            Ok(p)
        }
        None => Err(CachePathError::new(CachePathErrorType::UnkownCachePath)),
    }
}

async fn get_token(auth: &mut SpotifyOAuth) -> Option<TokenInfo> {
    match auth.get_cached_token().await {
        Some(token_info) => Some(token_info),
        None => generate_token(auth).await,
    }
}

async fn generate_token(auth: &mut SpotifyOAuth) -> Option<TokenInfo> {
    let state = generate_random_string(16);
    let auth_url = auth.get_authorize_url(Some(&state), None);
    let code_future = get_code_req(auth);
    webbrowser::open(&auth_url).unwrap();
    return code_future.await;
}

async fn get_code_req(auth: &mut SpotifyOAuth) -> Option<TokenInfo> {
    let listener = TcpListener::bind("127.0.0.1:29797").unwrap();

    for stream in listener.incoming() {
        let mut stream = stream.unwrap();
        let mut buffer = [0; 1024];

        stream.read(&mut buffer).unwrap();

        let mut req = String::from_utf8_lossy(&buffer[..]).to_string();
        match process_token(auth, &mut req).await {
            Some(token_info) => {
                let response = format!(
                    "HTTP/1.1 200 OK\r\n\r\n{}",
                    fs::read_to_string("static/auth-ok.html").unwrap(),
                );
                stream.write(response.as_bytes()).unwrap();
                stream.flush().unwrap();
                return Some(token_info);
            }
            None => {
                let response = format!(
                    "HTTP/1.1 400 BAD REQUEST\r\n\r\n{}",
                    fs::read_to_string("static/auth-error.html").unwrap(),
                );
                stream.write(response.as_bytes()).unwrap();
                stream.flush().unwrap();
            }
        }
    }

    None
}
