use serde::{Deserialize, Serialize};
use std::sync::Mutex;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AudioTrack {
    pub id: String,
    pub title: String,
    pub artist: String,
    pub duration_seconds: u32,
    pub source_url: String,
    pub is_downloaded: bool,
    pub download_progress: f32, // 0.0 to 100.0
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlayerState {
    pub current_track: Option<AudioTrack>,
    pub is_playing: bool,
    pub volume: f32,
    pub playlist: Vec<AudioTrack>,
    pub background_play_enabled: bool,
}

pub struct MediaManager {
    state: Mutex<PlayerState>,
}

impl MediaManager {
    pub fn new() -> Self {
        let sample_tracks = vec![
            AudioTrack {
                id: "track-1".to_string(),
                title: "Cyberpunk Ambient Stream (Dev Focus)".to_string(),
                artist: "Kestrel Audio".to_string(),
                duration_seconds: 320,
                source_url: "https://assets.kestrel.internal/audio/cyber_ambient.mp3".to_string(),
                is_downloaded: true,
                download_progress: 100.0,
            },
            AudioTrack {
                id: "track-2".to_string(),
                title: "Lofi Synthesizer Sessions".to_string(),
                artist: "SynthWave AI".to_string(),
                duration_seconds: 245,
                source_url: "https://assets.kestrel.internal/audio/lofi_synths.mp3".to_string(),
                is_downloaded: false,
                download_progress: 0.0,
            },
        ];

        Self {
            state: Mutex::new(PlayerState {
                current_track: sample_tracks.first().cloned(),
                is_playing: false,
                volume: 0.8,
                playlist: sample_tracks,
                background_play_enabled: true,
            }),
        }
    }

    pub fn get_state(&self) -> PlayerState {
        self.state.lock().unwrap().clone()
    }

    pub fn toggle_play(&self) -> bool {
        let mut state = self.state.lock().unwrap();
        state.is_playing = !state.is_playing;
        state.is_playing
    }

    pub fn trigger_download(&self, track_id: &str) -> Result<AudioTrack, String> {
        let mut state = self.state.lock().unwrap();
        if let Some(track) = state.playlist.iter_mut().find(|t| t.id == track_id) {
            track.is_downloaded = true;
            track.download_progress = 100.0;
            Ok(track.clone())
        } else {
            Err("Track not found".to_string())
        }
    }
}

#[tauri::command]
pub fn get_player_state(state: tauri::State<'_, MediaManager>) -> PlayerState {
    state.get_state()
}

#[tauri::command]
pub fn toggle_playback(state: tauri::State<'_, MediaManager>) -> bool {
    state.toggle_play()
}

#[tauri::command]
pub fn download_audio_stream(track_id: String, state: tauri::State<'_, MediaManager>) -> Result<AudioTrack, String> {
    state.trigger_download(&track_id)
}

