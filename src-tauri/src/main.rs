#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use serde::{Deserialize, Serialize};
use reqwest::Client;
use tauri::command;
use dotenv::dotenv;
use std::env;

// Response structure for Spotify tokens
#[derive(Serialize, Deserialize)]
struct TokenResponse {
    access_token: String,
    token_type: String,
    expires_in: u32,
    refresh_token: Option<String>,
    scope: String,
}

// Structure for Song data (you can modify it as needed)
#[derive(Serialize, Deserialize)]
struct Song {
    title: String,
    artist: String,
    image: String,
}

// Command to get the Spotify authorization URL
#[command]
fn get_spotify_auth_url() -> String {
    dotenv().ok(); // Load environment variables

    let client_id = env::var("SPOTIFY_CLIENT_ID").expect("SPOTIFY_CLIENT_ID not set");
    let redirect_uri = env::var("REDIRECT_URI").unwrap_or("http://localhost:3000/callback".to_string());
    let scopes = "user-read-playback-state user-read-currently-playing";

    format!(
        "https://accounts.spotify.com/authorize?client_id={}&response_type=code&redirect_uri={}&scope={}",
        client_id, redirect_uri, scopes
    )
}

// Command to exchange the Spotify authorization code for an access token
#[command]
async fn exchange_spotify_token(code: String) -> Result<TokenResponse, String> {
    dotenv().ok(); // Load environment variables

    let client_id = env::var("SPOTIFY_CLIENT_ID").expect("SPOTIFY_CLIENT_ID not set");
    let client_secret = env::var("SPOTIFY_CLIENT_SECRET").expect("SPOTIFY_CLIENT_SECRET not set");
    let redirect_uri = env::var("REDIRECT_URI").unwrap_or_else(|_| "http://localhost:3000/callback".to_string());

    let params = [
        ("grant_type", "authorization_code"),
        ("code", &code),
        ("redirect_uri", &redirect_uri),
        ("client_id", &client_id),
        ("client_secret", &client_secret),
    ];

    let client = Client::new();

    let response = client
        .post("https://accounts.spotify.com/api/token")
        .form(&params)
        .send()
        .await;

    match response {
        Ok(resp) => {
            if resp.status().is_success() {
                let token_data: TokenResponse = resp.json().await.unwrap();
                Ok(token_data)
            } else {
                Err(format!("Spotify error: {:?}", resp.text().await))
            }
        }
        Err(err) => Err(format!("Request failed: {:?}", err)),
    }
}


// Command to get the current song data from Spotify
#[command]
async fn get_current_song(access_token: String) -> Result<Song, String> {
    let client = reqwest::Client::new();
    
    // Log the access token for debugging
    println!("Received Access Token: {}", access_token);
    
    let response = client
        .get("https://api.spotify.com/v1/me/player/currently-playing")
        .bearer_auth(access_token)
        .send()
        .await;

    match response {
        Ok(resp) => {
            println!("Spotify API Response: {:?}", resp); // Log the raw response
            
            if resp.status().is_success() {
                let song_data: serde_json::Value = resp.json().await.unwrap();
                println!("Song Data: {:?}", song_data); // Log the song data

                let title = song_data["item"]["name"].as_str().unwrap_or("Unknown Song").to_string();
                let artist = song_data["item"]["artists"][0]["name"]
                    .as_str()
                    .unwrap_or("Unknown Artist")
                    .to_string();
                let image = song_data["item"]["album"]["images"][0]["url"]
                    .as_str()
                    .unwrap_or("https://via.placeholder.com/300")
                    .to_string();

                Ok(Song { title, artist, image })
            } else {
                Err(format!("Spotify error: {:?}", resp.text().await))
            }
        }
        Err(err) => Err(format!("Request failed: {:?}", err)),
    }
}




fn main() {
    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![get_spotify_auth_url, exchange_spotify_token, get_current_song])
        .run(tauri::generate_context!())
        .expect("error while running Tauri application");
}
