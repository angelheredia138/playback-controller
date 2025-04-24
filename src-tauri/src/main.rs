#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use serde::{Deserialize, Serialize};
use reqwest::Client;
use tauri::{ Manager}; // Ensure Manager is imported
use tauri::command;
use dotenv::dotenv;
use tauri::{Emitter};
use std::env;
use std::sync::Mutex;
use std::sync::Arc;
use hyper::{Body, Request, Response, Method, StatusCode};
use hyper::service::{make_service_fn, service_fn};
use url::Url; 

// Spotify Token Response
#[derive(Serialize, Deserialize, Debug)]
struct TokenResponse {
    access_token: String,
    token_type: String,
    expires_in: u32,
    refresh_token: Option<String>,
    scope: String,
}

// Song Data Structure
#[derive(Serialize, Deserialize, Debug)]
struct Song {
    title: String,
    artist: String,
    image: String,
    artist_image: String,
    progress_ms: u32,
    duration_ms: u32,
}


// State for storing the access token globally
struct AppState {
    access_token: Mutex<Option<String>>,
    auth_code: Mutex<Option<String>>,
}

#[command]
fn store_access_token(state: tauri::State<AppState>, token: String) -> Result<(), String> {
    let mut at = state.access_token.lock().map_err(|e| e.to_string())?;
    *at = Some(token);
    Ok(())
}

#[command]
fn get_auth_code(state: tauri::State<AppState>) -> Option<String> {
    state.auth_code.lock().unwrap().clone()
}

#[command]
async fn fetch_current_song(app: tauri::AppHandle, access: String) -> Result<Song, String> {

    let client = Client::new();
    let resp = client
        .get("https://api.spotify.com/v1/me/player/currently-playing")
        .bearer_auth(&access)
        .send()
        .await
        .map_err(|e| format!("Failed to reach Spotify API: {:?}", e))?;

    match resp.status() {
        reqwest::StatusCode::NO_CONTENT => {
            let message = "No song is currently playing.".to_string();
            app.emit("backend-log", message.clone()).unwrap();
            Err(message)
        }
        code if code.is_success() => {
            let json: serde_json::Value = resp
                .json()
                .await
                .map_err(|e| format!("Could not parse Spotify response: {:?}", e))?;

            
            let title = json["item"]["name"].as_str().unwrap_or("Unknown Title").to_string();
            let artist = json["item"]["artists"][0]["name"].as_str().unwrap_or("Unknown Artist").to_string();
            let album_image = json["item"]["album"]["images"][0]["url"]
                .as_str()
                .unwrap_or("https://via.placeholder.com/300")
                .to_string();
            let progress_ms = json["progress_ms"].as_u64().unwrap_or(0) as u32;
            let duration_ms = json["item"]["duration_ms"].as_u64().unwrap_or(0) as u32;


             // Extract the artist ID
             let artist_id = json["item"]["artists"][0]["id"]
             .as_str()
             .ok_or("No artist ID found.")?;

            let artist_resp = client
                .get(format!("https://api.spotify.com/v1/artists/{}", artist_id))
                .bearer_auth(&access)
                .send()
                .await
                .map_err(|e| format!("Failed to fetch artist info: {:?}", e))?;
                if artist_resp.status().is_success() {
                    let artist_data: serde_json::Value = artist_resp.json().await.map_err(|e| format!("Failed to parse artist JSON: {:?}", e))?;
                    let artist_image = artist_data["images"][0]["url"].as_str().unwrap_or("https://via.placeholder.com/300").to_string();
                    app.emit("backend-log", "Successfully fetched current song.".to_string()).unwrap();
                    Ok(Song {
                        title,
                        artist,
                        image: album_image,
                        artist_image,
                        progress_ms,
                        duration_ms,
                    })
                } else {
                    let error_text = artist_resp.text().await.unwrap_or("Unknown error".to_string());
                    Err(format!("Spotify API returned an error fetching artist: {}", error_text))
                }
        }
        _ => {
            let error_text = resp.text().await.unwrap_or("Unknown Spotify error".to_string());
            app.emit("backend-log", format!("Spotify API error: {}", error_text))
                .unwrap();
            Err(format!("Spotify API returned an error: {}", error_text))
        }
    }
}





#[command]
async fn exchange_spotify_token(code: String) -> Result<TokenResponse, String> {
    dotenv().ok();

    let client_id = env::var("SPOTIFY_CLIENT_ID").expect("SPOTIFY_CLIENT_ID not set");
    let client_secret = env::var("SPOTIFY_CLIENT_SECRET").expect("SPOTIFY_CLIENT_SECRET not set");
    let redirect_uri = env::var("REDIRECT_URI").unwrap_or("http://127.0.0.1:4242/callback".to_string());

    let params = [
        ("grant_type", "authorization_code"),
        ("code", &code),
        ("redirect_uri", &redirect_uri),
        ("client_id", &client_id),
        ("client_secret", &client_secret),
    ];

    let client = Client::new();
    let response = client.post("https://accounts.spotify.com/api/token").form(&params).send().await.map_err(|e| format!("Request failed: {:?}", e))?;

    if response.status().is_success() {
        let token_data: TokenResponse = response.json().await.map_err(|e| format!("JSON parse error: {:?}", e))?;
        Ok(token_data)
    } else {
        let error_text = response.text().await.unwrap_or("Unknown error".to_string());
        Err(format!("Spotify error: {:?}", error_text))
    }
}

#[command]
async fn refresh_spotify_token(refresh_token: String) -> Result<TokenResponse, String> {
    dotenv().ok();
    let client_id = env::var("SPOTIFY_CLIENT_ID").expect("SPOTIFY_CLIENT_ID not set");
    let client_secret = env::var("SPOTIFY_CLIENT_SECRET").expect("SPOTIFY_CLIENT_SECRET not set");

    let params = [
        ("grant_type", "refresh_token"),
        ("refresh_token", &refresh_token),
        ("client_id", &client_id),
        ("client_secret", &client_secret),
    ];

    let client = Client::new();
    let response = client.post("https://accounts.spotify.com/api/token").form(&params).send().await.map_err(|e| format!("Request failed: {:?}", e))?;

    if response.status().is_success() {
        let token_data: TokenResponse = response.json().await.map_err(|e| format!("JSON parse error: {:?}", e))?;
        Ok(token_data)
    } else {
        let error_text = response.text().await.unwrap_or("Unknown error".to_string());
        Err(format!("Spotify error: {:?}", error_text))
    }
}

#[command]
fn get_spotify_auth_url(app: tauri::AppHandle) -> String {
    dotenv().ok();
    app.emit("backend-log", "Generating Spotify Auth URL...").unwrap_or_else(|err| {
        eprintln!("Failed to emit log: {:?}", err);
    });

    let client_id = env::var("SPOTIFY_CLIENT_ID").expect("SPOTIFY_CLIENT_ID not set");
    let redirect_uri = env::var("REDIRECT_URI").unwrap_or("http://127.0.0.1:4242/callback".to_string());
    let scopes = "user-read-playback-state user-modify-playback-state streaming playlist-read-private playlist-read-collaborative";

    let auth_url = format!(
        "https://accounts.spotify.com/authorize?client_id={}&response_type=code&redirect_uri={}&scope={}",
        client_id, redirect_uri, scopes
    );

    app.emit("backend-log", format!("Generated Auth URL: {}", auth_url)).unwrap_or_else(|err| {
        eprintln!("Failed to emit log: {:?}", err);
    });

    auth_url
}

#[command]
async fn play(app: tauri::AppHandle, access: String) -> Result<(), String> {
    send_command("https://api.spotify.com/v1/me/player/play", &access, app).await
}


#[command]
async fn pause(app: tauri::AppHandle, access: String) -> Result<(), String> {
    send_command("https://api.spotify.com/v1/me/player/pause", &access, app).await
}


async fn send_command(endpoint: &str, access: &str, app: tauri::AppHandle) -> Result<(), String> {
    let client = Client::new();
    let resp = client
        .put(endpoint)
        .bearer_auth(access)
        .json(&serde_json::json!({})) // Add empty JSON body
        .send()
        .await
        .map_err(|e| format!("Failed to send command: {:?}", e))?;

    if resp.status().is_success() {
        app.emit(
            "backend-log",
            format!("Command to {} was successful.", endpoint),
        )
        .unwrap_or_else(|err| eprintln!("Failed to emit log: {:?}", err));
        Ok(())
    } else {
        let error_text = resp.text().await.unwrap_or("Unknown error".to_string());
        Err(format!("Spotify API error: {}", error_text))
    }
}


#[command]
async fn skip_next(app: tauri::AppHandle, access: String) -> Result<(), String> {
    let client = Client::new();
    let resp = client
        .post("https://api.spotify.com/v1/me/player/next")
        .bearer_auth(&access)
        .json(&serde_json::json!({})) // Explicitly include an empty JSON body
        .send()
        .await
        .map_err(|e| format!("Failed to send skip next command: {:?}", e))?;

    if resp.status().is_success() {
        app.emit("backend-log", "Successfully skipped to the next track.".to_string())
            .unwrap_or_else(|err| eprintln!("Failed to emit log: {:?}", err));
        Ok(())
    } else {
        let error_text = resp.text().await.unwrap_or("Unknown error".to_string());
        Err(format!("Spotify API error on skip next: {}", error_text))
    }
}


#[command]
async fn skip_previous(app: tauri::AppHandle, access: String) -> Result<(), String> {
    let client = Client::new();
    let resp = client
        .post("https://api.spotify.com/v1/me/player/previous")
        .bearer_auth(&access)
        .json(&serde_json::json!({})) // Explicitly include an empty JSON body
        .send()
        .await
        .map_err(|e| format!("Failed to send skip previous command: {:?}", e))?;

    if resp.status().is_success() {
        app.emit("backend-log", "Successfully skipped to the previous track.".to_string())
            .unwrap_or_else(|err| eprintln!("Failed to emit log: {:?}", err));
        Ok(())
    } else {
        let error_text = resp.text().await.unwrap_or("Unknown error".to_string());
        Err(format!("Spotify API error on skip previous: {}", error_text))
    }
}



#[command]
async fn toggle_shuffle(app: tauri::AppHandle, access: String) -> Result<bool, String> {
    let client = Client::new();

    // Check current shuffle state
    let playback_resp = client
        .get("https://api.spotify.com/v1/me/player")
        .bearer_auth(&access)
        .send()
        .await
        .map_err(|e| format!("Failed to get playback state: {:?}", e))?;

    if playback_resp.status().is_success() {
        let playback_data: serde_json::Value = playback_resp
            .json()
            .await
            .map_err(|e| format!("Failed to parse playback state: {:?}", e))?;

        let current_shuffle = playback_data
            .get("shuffle_state")
            .and_then(|v| v.as_bool())
            .unwrap_or(false);

        // Toggle shuffle state
        let new_shuffle_state = !current_shuffle;
        let toggle_resp = client
            .put(&format!(
                "https://api.spotify.com/v1/me/player/shuffle?state={}",
                new_shuffle_state
            ))
            .bearer_auth(&access)
            .json(&serde_json::json!({})) // Add empty JSON body
            .send()
            .await
            .map_err(|e| format!("Failed to toggle shuffle: {:?}", e))?;

        if toggle_resp.status().is_success() {
            app.emit(
                "backend-log",
                format!(
                    "Shuffle toggled. New state: {}",
                    if new_shuffle_state { "enabled" } else { "disabled" }
                ),
            )
            .unwrap_or_else(|err| eprintln!("Failed to emit log: {:?}", err));
            Ok(new_shuffle_state)
        } else {
            let error_text = toggle_resp
                .text()
                .await
                .unwrap_or("Unknown error".to_string());
            Err(format!("Spotify API error: {}", error_text))
        }
    } else {
        let error_text = playback_resp
            .text()
            .await
            .unwrap_or("Unknown error".to_string());
        Err(format!("Spotify API error: {}", error_text))
    }
}



#[command]
async fn restart_song(app: tauri::AppHandle, access: String) -> Result<(), String> {
    let client = Client::new();

    // Seek to the beginning of the current track (0 milliseconds)
    let seek_resp = client
        .put("https://api.spotify.com/v1/me/player/seek?position_ms=0")
        .bearer_auth(&access)
        .json(&serde_json::json!({})) // Add an empty JSON body
        .send()
        .await
        .map_err(|e| format!("Failed to restart song: {:?}", e))?;

    if seek_resp.status().is_success() {
        app.emit("backend-log", "Successfully restarted the current song.".to_string())
            .unwrap_or_else(|err| eprintln!("Failed to emit log: {:?}", err));
        Ok(())
    } else {
        let error_text = seek_resp.text().await.unwrap_or("Unknown error".to_string());
        Err(format!("Spotify API error: {}", error_text))
    }
}


#[command]
async fn fetch_playlists(app: tauri::AppHandle, access: String) -> Result<serde_json::Value, String> {
    let client = Client::new();
    let resp = client
        .get("https://api.spotify.com/v1/me/playlists")
        .bearer_auth(&access)
        .send()
        .await
        .map_err(|e| format!("Failed to fetch playlists: {:?}", e))?;

    if resp.status().is_success() {
        let playlists: serde_json::Value = resp.json().await.map_err(|e| format!("Failed to parse playlists: {:?}", e))?;
        app.emit("backend-log", "Playlists fetched successfully.".to_string())
            .unwrap_or_else(|err| eprintln!("Failed to emit log: {:?}", err));
        Ok(playlists)
    } else {
        let error_text = resp.text().await.unwrap_or("Unknown error".to_string());
        Err(format!("Spotify API error: {}", error_text))
    }
}



#[command]
async fn change_playlist(app: tauri::AppHandle, access: String, id: String) -> Result<(), String> {
    let client = reqwest::Client::new();

    let play_resp = client
        .put("https://api.spotify.com/v1/me/player/play")
        .bearer_auth(&access)
        .json(&serde_json::json!({ "context_uri": format!("spotify:playlist:{}", id) }))
        .send()
        .await
        .map_err(|e| format!("Failed to change playlist: {:?}", e))?;

    if play_resp.status().is_success() {
        app.emit(
            "backend-log",
            format!("Playlist successfully changed to ID: {}", id),
        )
        .unwrap_or_else(|err| eprintln!("Failed to emit log: {:?}", err));
        Ok(())
    } else {
        let error_text = play_resp.text().await.unwrap_or("Unknown error".to_string());
        Err(format!("Spotify API error: {}", error_text))
    }
}



#[command]
async fn set_volume(app: tauri::AppHandle, access: String, volume: u8) -> Result<(), String> {
    let client = Client::new();
    let endpoint = format!("https://api.spotify.com/v1/me/player/volume?volume_percent={}", volume);

    let resp = client
        .put(&endpoint)
        .bearer_auth(&access)
        .json(&serde_json::json!({})) // Add empty JSON body
        .send()
        .await
        .map_err(|e| format!("Failed to set volume: {:?}", e))?;

    if resp.status().is_success() {
        app.emit(
            "backend-log",
            format!("Volume set to {}%.", volume),
        )
        .unwrap_or_else(|err| eprintln!("Failed to emit log: {:?}", err));
        Ok(())
    } else {
        let error_text = resp.text().await.unwrap_or("Unknown error".to_string());
        Err(format!("Spotify API error: {}", error_text))
    }
}



#[command]
async fn get_devices(app: tauri::AppHandle, access: String) -> Result<serde_json::Value, String> {
    let client = Client::new();
    let resp = client
        .get("https://api.spotify.com/v1/me/player/devices")
        .bearer_auth(&access)
        .send()
        .await
        .map_err(|e| format!("Failed to fetch devices: {:?}", e))?;

    if resp.status().is_success() {
        let devices: serde_json::Value = resp.json().await.map_err(|e| format!("Failed to parse devices: {:?}", e))?;
        app.emit("backend-log", "Fetched available devices.".to_string())
            .unwrap_or_else(|err| eprintln!("Failed to emit log: {:?}", err));
        Ok(devices)
    } else {
        let error_text = resp.text().await.unwrap_or("Unknown error".to_string());
        Err(format!("Spotify API error: {}", error_text))
    }
}


#[command]
async fn get_playback_state(app: tauri::AppHandle, access: String) -> Result<serde_json::Value, String> {
    let client = Client::new();
    let resp = client
        .get("https://api.spotify.com/v1/me/player")
        .bearer_auth(&access)
        .send()
        .await
        .map_err(|e| format!("Failed to fetch playback state: {:?}", e))?;

    if resp.status().is_success() {
        let playback_data: serde_json::Value = resp
            .json()
            .await
            .map_err(|e| format!("Failed to parse playback state: {:?}", e))?;
        
        app.emit("backend-log", "Successfully fetched playback state.".to_string())
            .unwrap_or_else(|err| eprintln!("Failed to emit log: {:?}", err));

        Ok(playback_data)
    } else {
        let error_text = resp
            .text()
            .await
            .unwrap_or("Unknown error".to_string());
        Err(format!("Spotify API error: {}", error_text))
    }
}


#[command]
async fn get_current_playback(state: tauri::State<'_, AppState>) -> Result<serde_json::Value, String> {
    let access_token = {
        let guard = state.access_token.lock().map_err(|e| e.to_string())?;
        guard.clone().ok_or("No access token stored on backend.")?
    };

    let client = Client::new();
    let resp = client
        .get("https://api.spotify.com/v1/me/player")
        .bearer_auth(&access_token)
        .send()
        .await
        .map_err(|e| format!("Failed to fetch current playback: {:?}", e))?;

    if resp.status().is_success() {
        let playback_data: serde_json::Value = resp.json().await.map_err(|e| format!("Failed to parse playback state: {:?}", e))?;
        Ok(playback_data)
    } else {
        let error_text = resp.text().await.unwrap_or("Unknown error".to_string());
        Err(format!("Spotify API error: {}", error_text))
    }
}

#[command]
async fn toggle_fullscreen(app: tauri::AppHandle) -> Result<(), String> {
    // Use the Manager trait to access the main window
    if let Some(window) = app.get_webview_window("main") {
        let is_fullscreen = window.is_fullscreen().unwrap_or(false);
        window.set_fullscreen(!is_fullscreen).map_err(|e| e.to_string())?;
        app.emit("redraw", {}).map_err(|e| e.to_string())?;
        Ok(())
    } else {
        Err("Window 'main' not found.".to_string())
    }
}

// Fetch the user's Spotify profile (for profile image)
#[tauri::command]
async fn get_user_profile(access: String) -> Result<serde_json::Value, String> {
    let client = reqwest::Client::new();
    let resp = client
        .get("https://api.spotify.com/v1/me")
        .bearer_auth(&access)
        .send()
        .await
        .map_err(|e| format!("Failed to fetch user profile: {:?}", e))?;

    if resp.status().is_success() {
        let user: serde_json::Value = resp.json().await.map_err(|e| format!("Failed to parse user profile: {:?}", e))?;
        Ok(user)
    } else {
        let error_text = resp.text().await.unwrap_or("Unknown error".to_string());
        Err(format!("Spotify API error: {}", error_text))
    }
}

// Fetch the image URL for a given playlist
#[tauri::command]
async fn get_playlist_image(access: String, playlist_id: String) -> Result<String, String> {
    let client = reqwest::Client::new();
    let url = format!("https://api.spotify.com/v1/playlists/{}", playlist_id);
    let resp = client
        .get(&url)
        .bearer_auth(&access)
        .send()
        .await
        .map_err(|e| format!("Failed to fetch playlist: {:?}", e))?;

    if resp.status().is_success() {
        let playlist: serde_json::Value = resp.json().await.map_err(|e| format!("Failed to parse playlist: {:?}", e))?;
        let image_url = playlist["images"]
            .get(0)
            .and_then(|img| img["url"].as_str())
            .unwrap_or("https://placehold.co/600x600/222/fff?text=No+Image")
            .to_string();
        Ok(image_url)
    } else {
        let error_text = resp.text().await.unwrap_or("Unknown error".to_string());
        Err(format!("Spotify API error: {}", error_text))
    }
}

async fn callback_service(
    req: Request<Body>,
    app_state: Arc<AppState>,
) -> Result<Response<Body>, hyper::Error> {
    if req.method() == Method::GET && req.uri().path() == "/callback" {
        if let Some(query) = req.uri().query() {
            let url = Url::parse(&format!("http://127.0.0.1:4242/callback?{}", query))
                .unwrap_or_else(|_| Url::parse("http://127.0.0.1:4242/callback").unwrap());
            let code_param = url.query_pairs().find(|(k, _)| k == "code").map(|(_, v)| v.to_string());

            if let Some(code) = code_param {
                // Store the code in AppState for future use if needed
                if let Ok(mut auth_code) = app_state.auth_code.lock() {
                    *auth_code = Some(code.clone());
                }

                // Redirect to the frontend callback page with the code
                let body = format!(r#"<!DOCTYPE html>
<html>
<head>
<meta charset="UTF-8" />
<title>Authentication Complete</title>
<script>
    window.location.href = 'tauri://localhost/callback?code={code}';
</script>
</head>
<body>
</body>
</html>"#, code = code);
                return Ok(Response::new(Body::from(body)));
            }
        }

        // Return an error response if no code is provided
        let body = "Missing 'code' parameter.";
        return Ok(Response::builder()
            .status(StatusCode::BAD_REQUEST)
            .body(Body::from(body))
            .unwrap());
    }

    // If the path is not /callback, return a 404 Not Found
    Ok(Response::builder()
        .status(StatusCode::NOT_FOUND)
        .body(Body::from("Not Found"))
        .unwrap())
}


async fn start_server(app_state: Arc<AppState>) {
    let addr = ([127, 0, 0, 1], 4242).into();
    let make_svc = make_service_fn(move |_conn| {
        let state = app_state.clone();
        async move {
            Ok::<_, hyper::Error>(service_fn(move |req| {
                let state_inner = state.clone();
                async move { callback_service(req, state_inner).await }
            }))
        }
    });

    let server = hyper::Server::bind(&addr).serve(make_svc);
    if let Err(e) = server.await {
        eprintln!("server error: {}", e);
    }
}


fn main() {
    let app_state = Arc::new(AppState {
        access_token: Mutex::new(None),
        auth_code: Mutex::new(None), // NEW FIELD
    });

    tauri::Builder::default()
        .manage(app_state.clone()) // pass the Arc-managed state to Tauri
        .setup(move |_app| {
            // Start the local HTTP server in the background
            tauri::async_runtime::spawn(start_server(app_state));
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            get_spotify_auth_url,
            exchange_spotify_token,
            refresh_spotify_token,
            fetch_current_song,
            store_access_token,
            get_auth_code,      
            play,
            pause,
            skip_next,
            skip_previous,
            toggle_shuffle,
            restart_song,
            change_playlist,
            set_volume,
            get_devices,
            get_playback_state,
            fetch_playlists,
            get_current_playback,
            toggle_fullscreen,
            get_user_profile,
            get_playlist_image,
        ])
        .run(tauri::generate_context!())
        .expect("error while running Tauri application");
}
