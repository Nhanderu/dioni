use super::{
    args_parsing::ARGS,
    cond_print, cond_println,
    error_handling::{Error, Result},
    utils::clear_stdout_line,
};
use rspotify::{
    client::Spotify as SpotifyClient,
    oauth2::{SpotifyClientCredentials, SpotifyOAuth, TokenInfo},
    util::generate_random_string,
};
use std::{
    error, fs,
    io::prelude::*,
    net::{TcpListener, TcpStream},
    path::PathBuf,
    result,
};

pub async fn get_spotify_client(mut cache_path: PathBuf) -> Result<SpotifyClient> {
    cache_path.push(".spotify-token");
    let auth_cache_path = cache_path.to_path_buf();

    let mut auth = SpotifyOAuth::default()
        .redirect_uri("http://localhost:29797/")
        .client_id("ee0929df2d71455dbba55aeba1605e37")
        .client_secret("b840d375072840dcb4879b337e11c4ef")
        .cache_path(auth_cache_path)
        .scope("user-library-read user-modify-playback-state")
        .build();
    let creds = SpotifyClientCredentials::default()
        .token_info(get_token(&mut auth).await?)
        .build();
    Ok(SpotifyClient::default()
        .client_credentials_manager(creds)
        .build())
}

pub async fn play(
    client: SpotifyClient,
    tracks_uris: Vec<String>,
    queue_uris: Vec<String>,
) -> Result<()> {
    _play(client, tracks_uris, queue_uris).await?;
    Ok(())
}

async fn _play(
    client: SpotifyClient,
    tracks_uris: Vec<String>,
    queue_uris: Vec<String>,
) -> result::Result<(), Box<dyn error::Error>> {
    client.shuffle(false, None).await?;
    client
        .start_playback(None, None, Some(tracks_uris), None, None)
        .await?;
    cond_println!("Shuffle finished.");

    for (i, uri) in queue_uris.iter().enumerate() {
        clear_stdout_line();
        cond_print!(
            "{} out of {} tracks added to the queue.",
            i + 1,
            queue_uris.len()
        );
        client.add_item_to_queue(uri.to_string(), None).await?;
    }
    if queue_uris.len() > 0 {
        cond_println!();
    }

    Ok(())
}

async fn get_token(auth: &mut SpotifyOAuth) -> Result<TokenInfo> {
    if !ARGS.force_auth {
        match auth.get_cached_token().await {
            Some(token_info) => return Ok(token_info),
            None => {}
        };
    }
    let state = generate_random_string(16);
    let auth_url = auth.get_authorize_url(Some(&state), None);
    let code_future = get_code_req(auth, state);
    webbrowser::open(&auth_url)?;
    return code_future.await;
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

    Err(Error::AuthServerStopped)
}

fn make_http_response(stream: &mut TcpStream, ok: bool) -> Result<()> {
    let response = if ok {
        cond_println!("Authentication was validated.");
        format!(
            "HTTP/1.1 200 OK\r\n\r\n{}",
            fs::read_to_string("static/auth-ok.html")?,
        )
    } else {
        cond_println!("Invalid request for authentication. Please, try again.");
        format!(
            "HTTP/1.1 400 BAD REQUEST\r\n\r\n{}",
            fs::read_to_string("static/auth-error.html")?,
        )
    };
    stream.write(response.as_bytes())?;
    stream.flush()?;
    Ok(())
}
