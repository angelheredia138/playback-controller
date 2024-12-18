#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use serde::{Deserialize, Serialize};
use reqwest::Client;
use tauri::command;
use dotenv::dotenv;
use std::env;
use std::sync::Mutex;

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
}

// State for storing the access token globally
struct AppState {
    access_token: Mutex<Option<String>>,
}

#[command]
fn store_access_token(state: tauri::State<AppState>, token: String) -> Result<(), String> {
    let mut at = state.access_token.lock().map_err(|e| e.to_string())?;
    *at = Some(token);
    Ok(())
}

#[command]
async fn fetch_current_song(state: tauri::State<'_, AppState>) -> Result<Song, String> {
    let access_token = {
        let guard = state.access_token.lock().map_err(|e| e.to_string())?;
        guard.clone().ok_or("No access token stored on backend.")?
    };

    let client = Client::new();
    let resp = client
        .get("https://api.spotify.com/v1/me/player/currently-playing")
        .bearer_auth(&access_token)
        .send()
        .await
        .map_err(|e| format!("Failed to reach Spotify API: {:?}", e))?;

    match resp.status() {
        reqwest::StatusCode::NO_CONTENT => {
            Err("No song is currently playing.".to_string())
        }
        code if code.is_success() => {
            let json: serde_json::Value = resp.json().await.map_err(|e| format!("Could not parse Spotify response: {:?}", e))?;

            let title = json
                .get("item")
                .and_then(|item| item.get("name"))
                .and_then(|name| name.as_str())
                .unwrap_or("Unknown Title")
                .to_string();

            let artist = json
                .get("item")
                .and_then(|item| item.get("artists"))
                .and_then(|artists| artists.get(0))
                .and_then(|artist| artist.get("name"))
                .and_then(|name| name.as_str())
                .unwrap_or("Unknown Artist")
                .to_string();

            let image = json
                .get("item")
                .and_then(|item| item.get("album"))
                .and_then(|album| album.get("images"))
                .and_then(|images| images.get(0))
                .and_then(|img| img.get("url"))
                .and_then(|url| url.as_str())
                .unwrap_or("https://via.placeholder.com/300")
                .to_string();

            Ok(Song {
                title,
                artist,
                image,
            })
        }
        _ => {
            let error_text = resp.text().await.unwrap_or("Unknown Spotify error".to_string());
            Err(format!("Spotify API returned an error: {}", error_text))
        }
    }
}

#[command]
async fn exchange_spotify_token(code: String) -> Result<TokenResponse, String> {
    dotenv().ok();

    let client_id = env::var("SPOTIFY_CLIENT_ID").expect("SPOTIFY_CLIENT_ID not set");
    let client_secret = env::var("SPOTIFY_CLIENT_SECRET").expect("SPOTIFY_CLIENT_SECRET not set");
    let redirect_uri = env::var("REDIRECT_URI").unwrap_or("http://localhost:3000/callback".to_string());

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
fn get_spotify_auth_url() -> String {
    dotenv().ok();
    let client_id = env::var("SPOTIFY_CLIENT_ID").expect("SPOTIFY_CLIENT_ID not set");
    let redirect_uri = env::var("REDIRECT_URI").unwrap_or("http://localhost:3000/callback".to_string());
    // Add more scopes later if needed for full playback control
    let scopes = "user-read-playback-state user-read-currently-playing user-modify-playback-state";
    format!(
        "https://accounts.spotify.com/authorize?client_id={}&response_type=code&redirect_uri={}&scope={}",
        client_id, redirect_uri, scopes
    )
}

// Playback control commands (currently just placeholders returning Ok(()))
#[command]
async fn play_pause(state: tauri::State<'_, AppState>) -> Result<(), String> {
    let access_token = {
        let guard = state.access_token.lock().map_err(|e| e.to_string())?;
        guard.clone().ok_or("No access token stored on backend.")?
    };

    let client = Client::new();

    // Step 1: Get current playback state
    let player_state_resp = client
        .get("https://api.spotify.com/v1/me/player")
        .bearer_auth(&access_token)
        .send()
        .await
        .map_err(|e| format!("Failed to get player state: {:?}", e))?;

    if !player_state_resp.status().is_success() {
        let error_text = player_state_resp.text().await.unwrap_or("Unknown error".to_string());
        return Err(format!("Spotify API returned an error on get player state: {}", error_text));
    }

    let player_data: serde_json::Value = player_state_resp
        .json()
        .await
        .map_err(|e| format!("Failed to parse player state JSON: {:?}", e))?;

    let is_playing = player_data
        .get("is_playing")
        .and_then(|v| v.as_bool())
        .unwrap_or(false);

    // Step 2: Toggle play/pause based on current state
    let endpoint = if is_playing {
        "https://api.spotify.com/v1/me/player/pause"
    } else {
        "https://api.spotify.com/v1/me/player/play"
    };

    let toggle_resp = client
        .put(endpoint)
        .bearer_auth(&access_token)
        .json(&{}) // Send empty JSON to avoid 411 error
        .send()
        .await
        .map_err(|e| format!("Failed to send toggle command: {:?}", e))?;

    if toggle_resp.status().is_success() {
        Ok(())
    } else {
        let error_text = toggle_resp.text().await.unwrap_or("Unknown error".to_string());
        Err(format!("Spotify API returned an error on toggle: {}", error_text))
    }
}


#[command]
async fn skip_next(state: tauri::State<'_, AppState>) -> Result<(), String> {
    let _access_token = {
        let guard = state.access_token.lock().map_err(|e| e.to_string())?;
        guard.clone().ok_or("No access token stored on backend.")?
    };
    println!("Would have gone to next track.");
    Ok(())
}

#[command]
async fn skip_previous(state: tauri::State<'_, AppState>) -> Result<(), String> {
    let _access_token = {
        let guard = state.access_token.lock().map_err(|e| e.to_string())?;
        guard.clone().ok_or("No access token stored on backend.")?
    };
    println!("Would have gone to previous track.");
    Ok(())
}

#[command]
async fn toggle_shuffle(state: tauri::State<'_, AppState>) -> Result<(), String> {
    let _access_token = {
        let guard = state.access_token.lock().map_err(|e| e.to_string())?;
        guard.clone().ok_or("No access token stored on backend.")?
    };
    println!("Would have toggled shuffle.");
    Ok(())
}

#[command]
async fn restart_song(state: tauri::State<'_, AppState>) -> Result<(), String> {
    let _access_token = {
        let guard = state.access_token.lock().map_err(|e| e.to_string())?;
        guard.clone().ok_or("No access token stored on backend.")?
    };
    println!("Would have restarted the song.");
    Ok(())
}

#[command]
async fn change_playlist(state: tauri::State<'_, AppState>) -> Result<(), String> {
    let _access_token = {
        let guard = state.access_token.lock().map_err(|e| e.to_string())?;
        guard.clone().ok_or("No access token stored on backend.")?
    };
    println!("Would have changed the playlist.");
    Ok(())
}

#[command]
async fn set_volume(state: tauri::State<'_, AppState>, volume: u8) -> Result<(), String> {
    let _access_token = {
        let guard = state.access_token.lock().map_err(|e| e.to_string())?;
        guard.clone().ok_or("No access token stored on backend.")?
    };
    println!("Would have set volume to {}.", volume);
    Ok(())
}

fn main() {
    tauri::Builder::default()
        .manage(AppState {
            access_token: Mutex::new(None)
        })
        .invoke_handler(tauri::generate_handler![
            get_spotify_auth_url,
            exchange_spotify_token,
            refresh_spotify_token,
            fetch_current_song,
            store_access_token,
            play_pause,
            skip_next,
            skip_previous,
            toggle_shuffle,
            restart_song,
            change_playlist,
            set_volume
        ])
        .run(tauri::generate_context!())
        .expect("error while running Tauri application");
}
