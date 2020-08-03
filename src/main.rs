mod error;

extern crate dirs_next;
extern crate rspotify;

use std::env;
use std::error::Error;
use std::fs;
use std::io::prelude::*;
use std::net::{TcpListener, TcpStream};
use std::result::Result as StdResult;

use std::path::PathBuf;

use error::{DioniError, DioniErrorType, Result};

use rspotify::client::Spotify;
use rspotify::oauth2::{SpotifyClientCredentials, SpotifyOAuth, TokenInfo};
use rspotify::util::generate_random_string;

#[tokio::main]
async fn main() -> StdResult<(), Box<dyn Error>> {
    let cache_path = get_cache_path()?;
    let mut auth = SpotifyOAuth::default()
        .redirect_uri("http://localhost:29797/")
        .cache_path(cache_path)
        .build();
    match get_token(&mut auth).await {
        Ok(token_info) => {
            let creds = SpotifyClientCredentials::default()
                .token_info(token_info)
                .build();
            let spotify = Spotify::default().client_credentials_manager(creds).build();
            let me = spotify.me().await;
            println!("{:?}", me?);
        }
        Err(e) => eprintln!("{}", e),
    };
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
            p.push(".spotify_token_cache.json");
            Ok(p)
        }
        None => Err(DioniError::new(DioniErrorType::UnkownCachePath)),
    }
}

async fn get_token(auth: &mut SpotifyOAuth) -> Result<TokenInfo> {
    match auth.get_cached_token().await {
        Some(token_info) => Ok(token_info),
        None => {
            let state = generate_random_string(16);
            let auth_url = auth.get_authorize_url(Some(&state), None);
            let code_future = get_code_req(auth, state);
            webbrowser::open(&auth_url)?;
            return code_future.await;
        }
    }
}

async fn get_code_req(auth: &mut SpotifyOAuth, correct_state: String) -> Result<TokenInfo> {
    let listener = TcpListener::bind("127.0.0.1:29797")?;
    for stream in listener.incoming() {
        let mut stream = stream?;
        let mut buffer = [0; 1024];

        stream.read(&mut buffer)?;
        let req = String::from_utf8_lossy(&buffer[..]).to_string();

        let mut code: Option<String> = None;
        let mut state: Option<String> = None;
        req.lines().nth(0).map(|header| {
            header.split("?").nth(1).map(|header| {
                header.split(" ").nth(0).map(|queries| {
                    for query in queries.split("&") {
                        let mut s = query.split("=");
                        if let (Some(k), Some(v)) = (s.next(), s.next()) {
                            match k {
                                "code" => code = Some(v.to_string()),
                                "state" => state = Some(v.to_string()),
                                _ => {}
                            }
                        }
                    }
                })
            })
        });

        if let (Some(code), Some(state)) = (code, state) {
            if state != correct_state {
                make_http_response(&mut stream, false)?;
                continue;
            }

            match auth.get_access_token(&code).await {
                Some(token_info) => {
                    make_http_response(&mut stream, true)?;
                    return Ok(token_info);
                }
                None => {
                    make_http_response(&mut stream, false)?;
                }
            }
        }
    }

    Err(DioniError::new(DioniErrorType::AuthServerStopped))
}

fn make_http_response(stream: &mut TcpStream, ok: bool) -> Result<()> {
    let response = if ok {
        format!(
            "HTTP/1.1 200 OK\r\n\r\n{}",
            fs::read_to_string("static/auth-ok.html")?,
        )
    } else {
        format!(
            "HTTP/1.1 400 BAD REQUEST\r\n\r\n{}",
            fs::read_to_string("static/auth-error.html")?,
        )
    };
    stream.write(response.as_bytes())?;
    stream.flush()?;
    Ok(())
}
