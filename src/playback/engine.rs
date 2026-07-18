use async_trait::async_trait;
use std::time::{Duration, Instant};

use crate::config::Config;
use crate::errors::*;
use crate::types::*;

#[async_trait]
pub trait PlaybackEngine: Send + Sync {
    async fn play(&mut self, track_id: &TrackId, stream_url: &str) -> Result<()>;
    async fn pause(&mut self) -> Result<()>;
    async fn resume(&mut self) -> Result<()>;
    async fn stop(&mut self) -> Result<()>;
    async fn seek(&mut self, position: Duration) -> Result<()>;
    fn set_volume(&mut self, volume: f64);
    fn volume(&self) -> f64;
    fn status(&self) -> PlaybackStatus;
    fn position(&self) -> Duration;
    fn duration(&self) -> Duration;
    fn is_muted(&self) -> bool;
    fn set_mute(&mut self, muted: bool);
}

pub struct MockPlaybackEngine {
    status: PlaybackStatus,
    paused_position: Duration,
    duration: Duration,
    volume: f64,
    muted: bool,
    current_track_id: Option<TrackId>,
    play_start: Option<Instant>,
}

impl Default for MockPlaybackEngine {
    fn default() -> Self {
        Self::new()
    }
}

impl MockPlaybackEngine {
    pub fn new() -> Self {
        Self {
            status: PlaybackStatus::Stopped,
            paused_position: Duration::ZERO,
            duration: Duration::from_secs(240),
            volume: 0.8,
            muted: false,
            current_track_id: None,
            play_start: None,
        }
    }

    fn elapsed(&self) -> Duration {
        self.play_start
            .map(|start| start.elapsed())
            .unwrap_or(Duration::ZERO)
    }
}

#[async_trait]
impl PlaybackEngine for MockPlaybackEngine {
    async fn play(&mut self, track_id: &TrackId, _stream_url: &str) -> Result<()> {
        self.status = PlaybackStatus::Playing;
        self.paused_position = Duration::ZERO;
        self.duration = Duration::from_secs(240);
        self.current_track_id = Some(track_id.clone());
        self.play_start = Some(Instant::now());
        Ok(())
    }

    async fn pause(&mut self) -> Result<()> {
        match self.status {
            PlaybackStatus::Playing => {
                self.status = PlaybackStatus::Paused;
                self.paused_position = self.elapsed().min(self.duration);
                self.play_start = None;
                Ok(())
            }
            _ => Err(SymphonyError::playback("Cannot pause: not playing")),
        }
    }

    async fn resume(&mut self) -> Result<()> {
        match self.status {
            PlaybackStatus::Paused => {
                self.status = PlaybackStatus::Playing;
                self.play_start = Some(Instant::now());
                Ok(())
            }
            _ => Err(SymphonyError::playback("Cannot resume: not paused")),
        }
    }

    async fn stop(&mut self) -> Result<()> {
        self.status = PlaybackStatus::Stopped;
        self.paused_position = Duration::ZERO;
        self.current_track_id = None;
        self.play_start = None;
        Ok(())
    }

    async fn seek(&mut self, position: Duration) -> Result<()> {
        if position > self.duration {
            return Err(SymphonyError::playback(format!(
                "Seek position {:?} exceeds duration {:?}",
                position, self.duration
            )));
        }
        match self.status {
            PlaybackStatus::Playing => {
                self.play_start = Some(Instant::now());
                self.paused_position = position;
            }
            _ => {
                self.paused_position = position;
            }
        }
        Ok(())
    }

    fn set_volume(&mut self, volume: f64) {
        self.volume = volume.clamp(0.0, 1.0);
    }

    fn volume(&self) -> f64 {
        self.volume
    }

    fn status(&self) -> PlaybackStatus {
        if self.status == PlaybackStatus::Playing && self.elapsed() >= self.duration {
            PlaybackStatus::Stopped
        } else {
            self.status
        }
    }

    fn position(&self) -> Duration {
        match self.status {
            PlaybackStatus::Playing => {
                let pos = self.paused_position + self.elapsed();
                pos.min(self.duration)
            }
            PlaybackStatus::Paused => self.paused_position,
            PlaybackStatus::Stopped => Duration::ZERO,
        }
    }

    fn duration(&self) -> Duration {
        self.duration
    }

    fn is_muted(&self) -> bool {
        self.muted
    }

    fn set_mute(&mut self, muted: bool) {
        self.muted = muted;
    }
}

pub fn create_playback_engine(_config: &Config) -> Box<dyn PlaybackEngine + Send> {
    Box::new(MockPlaybackEngine::new())
}
