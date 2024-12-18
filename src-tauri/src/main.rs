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
    artist_image: String,
    progress_ms: u32,
    duration_ms: u32,
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

            let title = json["item"]["name"].as_str().unwrap_or("Unknown Title").to_string();
            let artist = json["item"]["artists"][0]["name"].as_str().unwrap_or("Unknown Artist").to_string();
            let album_image = json["item"]["album"]["images"][0]["url"].as_str().unwrap_or("https://via.placeholder.com/300").to_string();

            let progress_ms = json["progress_ms"].as_u64().unwrap_or(0) as u32;
            let duration_ms = json["item"]["duration_ms"].as_u64().unwrap_or(0) as u32;

            // Extract the artist ID
            let artist_id = json["item"]["artists"][0]["id"]
                .as_str()
                .ok_or("No artist ID found.")?;

            // Fetch artist details
            let artist_resp = client
                .get(format!("https://api.spotify.com/v1/artists/{}", artist_id))
                .bearer_auth(&access_token)
                .send()
                .await
                .map_err(|e| format!("Failed to fetch artist info: {:?}", e))?;

            if artist_resp.status().is_success() {
                let artist_data: serde_json::Value = artist_resp.json().await.map_err(|e| format!("Failed to parse artist JSON: {:?}", e))?;
                let artist_image = artist_data["images"][0]["url"].as_str().unwrap_or("https://via.placeholder.com/300").to_string();

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
    let scopes = "user-read-playback-state user-modify-playback-state streaming playlist-read-private playlist-read-collaborative";
    format!(
        "https://accounts.spotify.com/authorize?client_id={}&response_type=code&redirect_uri={}&scope={}",
        client_id, redirect_uri, scopes
    )
}


#[command]
async fn play(state: tauri::State<'_, AppState>) -> Result<(), String> {
    let access_token = {
        let guard = state.access_token.lock().map_err(|e| e.to_string())?;
        guard.clone().ok_or("No access token stored on backend.")?
    };

    let client = Client::new();

    // Get available devices
    let devices_resp = client
        .get("https://api.spotify.com/v1/me/player/devices")
        .bearer_auth(&access_token)
        .send()
        .await
        .map_err(|e| format!("Failed to fetch devices: {:?}", e))?;

    let devices: serde_json::Value = devices_resp
        .json()
        .await
        .map_err(|e| format!("Failed to parse devices: {:?}", e))?;

    let device_id = devices
        .get("devices")
        .and_then(|devices| devices.as_array())
        .and_then(|devices| devices.get(0)) // Pick the first available device
        .and_then(|device| device.get("id"))
        .and_then(|id| id.as_str())
        .ok_or("No active devices found. Open Spotify on a device.")?;

    // Start playback
    let resp = client
        .put("https://api.spotify.com/v1/me/player/play")
        .bearer_auth(&access_token)
        .json(&serde_json::json!({ "device_id": device_id }))
        .send()
        .await
        .map_err(|e| format!("Failed to send play command: {:?}", e))?;

    if resp.status().is_success() {
        Ok(())
    } else {
        let error_text = resp.text().await.unwrap_or("Unknown error".to_string());
        Err(format!("Spotify API error: {}", error_text))
    }
}


#[command]
async fn pause(state: tauri::State<'_, AppState>) -> Result<(), String> {
    send_command("https://api.spotify.com/v1/me/player/pause", state).await
}

async fn send_command(endpoint: &str, state: tauri::State<'_, AppState>) -> Result<(), String> {
    let access_token = {
        let guard = state.access_token.lock().map_err(|e| e.to_string())?;
        guard.clone().ok_or("No access token stored on backend.")?
    };

    let client = Client::new();
    let resp = client
        .put(endpoint)
        .bearer_auth(&access_token)
        .json(&{})
        .send()
        .await
        .map_err(|e| format!("Failed to send command: {:?}", e))?;

    if resp.status().is_success() {
        Ok(())
    } else {
        let error_text = resp.text().await.unwrap_or("Unknown error".to_string());
        Err(format!("Spotify API error: {}", error_text))
    }
}

#[command]
async fn skip_next(state: tauri::State<'_, AppState>) -> Result<(), String> {
    let access_token = {
        let guard = state.access_token.lock().map_err(|e| e.to_string())?;
        guard.clone().ok_or("No access token stored on backend.")?
    };

    let client = Client::new();
    let resp = client
        .post("https://api.spotify.com/v1/me/player/next")
        .bearer_auth(&access_token)
        .json(&serde_json::json!({})) // Explicitly include an empty JSON body
        .send()
        .await
        .map_err(|e| format!("Failed to send skip next command: {:?}", e))?;

    if resp.status().is_success() {
        Ok(())
    } else {
        let error_text = resp.text().await.unwrap_or("Unknown error".to_string());
        Err(format!("Spotify API error on skip next: {}", error_text))
    }
}


#[command]
async fn skip_previous(state: tauri::State<'_, AppState>) -> Result<(), String> {
    let access_token = {
        let guard = state.access_token.lock().map_err(|e| e.to_string())?;
        guard.clone().ok_or("No access token stored on backend.")?
    };

    let client = Client::new();
    let resp = client
        .post("https://api.spotify.com/v1/me/player/previous")
        .bearer_auth(&access_token)
        .json(&serde_json::json!({})) // Explicitly include an empty JSON body
        .send()
        .await
        .map_err(|e| format!("Failed to send skip previous command: {:?}", e))?;

    if resp.status().is_success() {
        Ok(())
    } else {
        let error_text = resp.text().await.unwrap_or("Unknown error".to_string());
        Err(format!("Spotify API error on skip previous: {}", error_text))
    }
}


#[command]
async fn toggle_shuffle(state: tauri::State<'_, AppState>) -> Result<bool, String> {
    let access_token = {
        let guard = state.access_token.lock().map_err(|e| e.to_string())?;
        guard.clone().ok_or("No access token stored on backend.")?
    };

    let client = Client::new();

    // Check current shuffle state
    let playback_resp = client
        .get("https://api.spotify.com/v1/me/player")
        .bearer_auth(&access_token)
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
            .bearer_auth(&access_token)
            .json(&serde_json::json!({})) // Add empty JSON body
            .send()
            .await
            .map_err(|e| format!("Failed to toggle shuffle: {:?}", e))?;

        if toggle_resp.status().is_success() {
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
async fn restart_song(state: tauri::State<'_, AppState>) -> Result<(), String> {
    let access_token = {
        let guard = state.access_token.lock().map_err(|e| e.to_string())?;
        guard.clone().ok_or("No access token stored on backend.")?
    };

    let client = Client::new();

    // Seek to the beginning of the current track (0 milliseconds)
    let seek_resp = client
        .put("https://api.spotify.com/v1/me/player/seek?position_ms=0")
        .bearer_auth(&access_token)
        .json(&serde_json::json!({})) // Add an empty JSON body
        .send()
        .await
        .map_err(|e| format!("Failed to restart song: {:?}", e))?;

    if seek_resp.status().is_success() {
        Ok(())
    } else {
        let error_text = seek_resp.text().await.unwrap_or("Unknown error".to_string());
        Err(format!("Spotify API error: {}", error_text))
    }
}

#[command]
async fn fetch_playlists(state: tauri::State<'_, AppState>) -> Result<serde_json::Value, String> {
    let access_token = {
        let guard = state.access_token.lock().map_err(|e| e.to_string())?;
        guard.clone().ok_or("No access token stored on backend.")?
    };

    let client = Client::new();
    let resp = client
        .get("https://api.spotify.com/v1/me/playlists")
        .bearer_auth(&access_token)
        .send()
        .await
        .map_err(|e| format!("Failed to fetch playlists: {:?}", e))?;

    if resp.status().is_success() {
        let playlists: serde_json::Value = resp.json().await.map_err(|e| format!("Failed to parse playlists: {:?}", e))?;
        Ok(playlists)
    } else {
        let error_text = resp.text().await.unwrap_or("Unknown error".to_string());
        Err(format!("Spotify API error: {}", error_text))
    }
}


#[command]
async fn change_playlist(state: tauri::State<'_, AppState>, playlist_id: String) -> Result<(), String> {
    
    let access_token = {
        let guard = state.access_token.lock().map_err(|e| e.to_string())?;
        guard.clone().ok_or("No access token stored on backend.")?
    };

    let client = reqwest::Client::new();

    let play_resp = client
        .put("https://api.spotify.com/v1/me/player/play")
        .bearer_auth(&access_token)
        .json(&serde_json::json!({ "context_uri": format!("spotify:playlist:{}", playlist_id) }))
        .send()
        .await
        .map_err(|e| format!("Failed to change playlist: {:?}", e))?;

    if play_resp.status().is_success() {
        Ok(())
    } else {
        let error_text = play_resp.text().await.unwrap_or("Unknown error".to_string());
        Err(format!("Spotify API error: {}", error_text))
    }
}


#[command]
async fn set_volume(state: tauri::State<'_, AppState>, volume: u8) -> Result<(), String> {
    let access_token = {
        let guard = state.access_token.lock().map_err(|e| e.to_string())?;
        guard.clone().ok_or("No access token stored on backend.")?
    };

    let client = Client::new();
    let endpoint = format!("https://api.spotify.com/v1/me/player/volume?volume_percent={}", volume);

    let resp = client
        .put(&endpoint)
        .bearer_auth(&access_token)
        // Even though we don't need a body, sending empty JSON ensures Content-Length is present
        .json(&serde_json::json!({}))
        .send()
        .await
        .map_err(|e| format!("Failed to set volume: {:?}", e))?;

    if resp.status().is_success() {
        
        Ok(())
    } else {
        let error_text = resp.text().await.unwrap_or("Unknown error".to_string());
        Err(format!("Spotify API error: {}", error_text))
    }
}


#[command]
async fn get_devices(state: tauri::State<'_, AppState>) -> Result<serde_json::Value, String> {
    let access_token = {
        let guard = state.access_token.lock().map_err(|e| e.to_string())?;
        guard.clone().ok_or("No access token stored on backend.")?
    };

    let client = Client::new();
    let resp = client
        .get("https://api.spotify.com/v1/me/player/devices")
        .bearer_auth(&access_token)
        .send()
        .await
        .map_err(|e| format!("Failed to fetch devices: {:?}", e))?;

    if resp.status().is_success() {
        let devices: serde_json::Value = resp.json().await.map_err(|e| format!("Failed to parse devices: {:?}", e))?;
        Ok(devices)
    } else {
        let error_text = resp.text().await.unwrap_or("Unknown error".to_string());
        Err(format!("Spotify API error: {}", error_text))
    }
}

#[command]
async fn get_playback_state(state: tauri::State<'_, AppState>) -> Result<serde_json::Value, String> {
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
        .map_err(|e| format!("Failed to fetch playback state: {:?}", e))?;

    if resp.status().is_success() {
        let playback_data: serde_json::Value = resp
            .json()
            .await
            .map_err(|e| format!("Failed to parse playback state: {:?}", e))?;

        // Ensure shuffle_state is part of the response
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
        ])
        .run(tauri::generate_context!())
        .expect("error while running Tauri application");
}
