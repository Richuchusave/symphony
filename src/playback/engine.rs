use async_trait::async_trait;
use rodio::{Decoder, DeviceSinkBuilder, MixerDeviceSink, Player, Source};
use std::fs::File;
use std::io::{Cursor, Write};
use std::os::unix::net::UnixStream;
use std::path::PathBuf;
use std::process::{Child, Command, Stdio};
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

/// Audio playback through the system's default output device.
///
/// The output device is opened lazily on the first track. This lets Symphony
/// start normally on headless systems while still returning a useful error if
/// playback is attempted without an available audio device.
pub struct RodioPlaybackEngine {
    output: Option<MixerDeviceSink>,
    player: Option<Player>,
    status: PlaybackStatus,
    duration: Duration,
    volume: f64,
    muted: bool,
    current_track_id: Option<TrackId>,
}

impl Default for RodioPlaybackEngine {
    fn default() -> Self {
        Self::new()
    }
}

impl RodioPlaybackEngine {
    pub fn new() -> Self {
        Self {
            output: None,
            player: None,
            status: PlaybackStatus::Stopped,
            duration: Duration::ZERO,
            volume: 0.8,
            muted: false,
            current_track_id: None,
        }
    }

    fn ensure_output(&mut self) -> Result<()> {
        if self.output.is_none() {
            let mut output = DeviceSinkBuilder::open_default_sink().map_err(|error| {
                SymphonyError::playback(format!("Could not open the default audio output: {error}"))
            })?;
            output.log_on_drop(false);
            self.output = Some(output);
        }
        Ok(())
    }

    fn new_player(&mut self) -> Result<Player> {
        self.ensure_output()?;
        if let Some(player) = self.player.take() {
            player.stop();
        }
        let player = Player::connect_new(
            self.output
                .as_ref()
                .expect("audio output was initialized")
                .mixer(),
        );
        player.set_volume(if self.muted { 0.0 } else { self.volume as f32 });
        Ok(player)
    }

    fn demo_frequency(track_id: &TrackId) -> f32 {
        let hash = track_id.bytes().fold(0_u32, |value, byte| {
            value.wrapping_mul(31).wrapping_add(byte as u32)
        });
        220.0 + (hash % 260) as f32
    }
}

#[async_trait]
impl PlaybackEngine for RodioPlaybackEngine {
    async fn play(&mut self, track_id: &TrackId, stream_url: &str) -> Result<()> {
        let player = self.new_player()?;

        if stream_url.starts_with("https://mock.stream/") {
            // The built-in catalog has no remote media. Produce a quiet tone so
            // it remains useful for testing the complete audio path.
            self.duration = Duration::from_secs(240);
            let source = rodio::source::SineWave::new(Self::demo_frequency(track_id))
                .take_duration(self.duration)
                .amplify(0.12);
            player.append(source);
        } else if let Some(path) = stream_url.strip_prefix("file://") {
            let file = File::open(path).map_err(|error| {
                SymphonyError::playback(format!("Could not open '{path}': {error}"))
            })?;
            let source = Decoder::try_from(file).map_err(|error| {
                SymphonyError::playback(format!("Could not decode '{path}': {error}"))
            })?;
            self.duration = source.total_duration().unwrap_or(Duration::ZERO);
            player.append(source);
        } else if stream_url.starts_with("http://") || stream_url.starts_with("https://") {
            let response = reqwest::get(stream_url)
                .await
                .map_err(|error| {
                    SymphonyError::playback(format!("Could not download track: {error}"))
                })?
                .error_for_status()
                .map_err(|error| {
                    SymphonyError::playback(format!("Could not download track: {error}"))
                })?;
            let bytes = response.bytes().await.map_err(|error| {
                SymphonyError::playback(format!("Could not read track: {error}"))
            })?;
            let source = Decoder::try_from(Cursor::new(bytes.to_vec())).map_err(|error| {
                SymphonyError::playback(format!("Could not decode downloaded track: {error}"))
            })?;
            self.duration = source.total_duration().unwrap_or(Duration::ZERO);
            player.append(source);
        } else {
            return Err(SymphonyError::playback(format!(
                "Unsupported stream URL: {stream_url}"
            )));
        }

        self.player = Some(player);
        self.status = PlaybackStatus::Playing;
        self.current_track_id = Some(track_id.clone());
        Ok(())
    }

    async fn pause(&mut self) -> Result<()> {
        let player = self
            .player
            .as_ref()
            .ok_or_else(|| SymphonyError::playback("Cannot pause: not playing"))?;
        player.pause();
        self.status = PlaybackStatus::Paused;
        Ok(())
    }

    async fn resume(&mut self) -> Result<()> {
        let player = self
            .player
            .as_ref()
            .ok_or_else(|| SymphonyError::playback("Cannot resume: not paused"))?;
        player.play();
        self.status = PlaybackStatus::Playing;
        Ok(())
    }

    async fn stop(&mut self) -> Result<()> {
        if let Some(player) = self.player.take() {
            player.stop();
        }
        self.status = PlaybackStatus::Stopped;
        self.current_track_id = None;
        Ok(())
    }

    async fn seek(&mut self, position: Duration) -> Result<()> {
        self.player
            .as_ref()
            .ok_or_else(|| SymphonyError::playback("Cannot seek: not playing"))?
            .try_seek(position)
            .map_err(|error| SymphonyError::playback(format!("Could not seek: {error}")))
    }

    fn set_volume(&mut self, volume: f64) {
        self.volume = volume.clamp(0.0, 1.0);
        if !self.muted {
            if let Some(player) = &self.player {
                player.set_volume(self.volume as f32);
            }
        }
    }

    fn volume(&self) -> f64 {
        self.volume
    }

    fn status(&self) -> PlaybackStatus {
        match &self.player {
            Some(player) if player.empty() => PlaybackStatus::Stopped,
            _ => self.status,
        }
    }

    fn position(&self) -> Duration {
        self.player
            .as_ref()
            .map(Player::get_pos)
            .unwrap_or(Duration::ZERO)
    }

    fn duration(&self) -> Duration {
        self.duration
    }

    fn is_muted(&self) -> bool {
        self.muted
    }

    fn set_mute(&mut self, muted: bool) {
        self.muted = muted;
        if let Some(player) = &self.player {
            player.set_volume(if muted { 0.0 } else { self.volume as f32 });
        }
    }
}

/// Streaming playback controlled through mpv's local JSON IPC socket.
pub struct MpvPlaybackEngine {
    child: Option<Child>,
    ipc_path: PathBuf,
    status: PlaybackStatus,
    paused_position: Duration,
    duration: Duration,
    volume: f64,
    muted: bool,
    current_track_id: Option<TrackId>,
    play_start: Option<Instant>,
    yt_dlp_path: PathBuf,
}

impl Default for MpvPlaybackEngine {
    fn default() -> Self {
        Self::new()
    }
}

impl MpvPlaybackEngine {
    pub fn new() -> Self {
        let runtime_dir = std::env::var_os("XDG_RUNTIME_DIR")
            .map(PathBuf::from)
            .unwrap_or_else(std::env::temp_dir);
        let yt_dlp_path = dirs::home_dir()
            .map(|home| home.join(".local/bin/yt-dlp"))
            .filter(|path| path.is_file())
            .unwrap_or_else(|| PathBuf::from("yt-dlp"));

        Self {
            child: None,
            ipc_path: runtime_dir.join(format!("symphony-mpv-{}.sock", std::process::id())),
            status: PlaybackStatus::Stopped,
            paused_position: Duration::ZERO,
            duration: Duration::ZERO,
            volume: 0.8,
            muted: false,
            current_track_id: None,
            play_start: None,
            yt_dlp_path,
        }
    }

    fn media_url(stream_url: &str) -> Result<(String, Duration, bool)> {
        if let Some(spec) = stream_url.strip_prefix("youtube://") {
            let (video_id, query) = spec.split_once('?').unwrap_or((spec, ""));
            if video_id.is_empty()
                || !video_id
                    .chars()
                    .all(|character| character.is_ascii_alphanumeric() || "-_".contains(character))
            {
                return Err(SymphonyError::playback("Invalid YouTube video id"));
            }
            let duration = query
                .split('&')
                .find_map(|part| part.strip_prefix("duration="))
                .and_then(|seconds| seconds.parse::<u64>().ok())
                .map(Duration::from_secs)
                .unwrap_or(Duration::ZERO);
            return Ok((
                format!("https://www.youtube.com/watch?v={video_id}"),
                duration,
                false,
            ));
        }

        if stream_url.starts_with("https://mock.stream/") {
            return Ok((
                "av://lavfi:sine=frequency=440".to_string(),
                Duration::from_secs(240),
                true,
            ));
        }

        if stream_url.starts_with("file://")
            || stream_url.starts_with("http://")
            || stream_url.starts_with("https://")
        {
            return Ok((stream_url.to_string(), Duration::ZERO, false));
        }

        Err(SymphonyError::playback(format!(
            "Unsupported stream URL: {stream_url}"
        )))
    }

    fn elapsed(&self) -> Duration {
        self.play_start
            .map(|start| start.elapsed())
            .unwrap_or(Duration::ZERO)
    }

    fn send_command(&self, command: serde_json::Value) -> Result<()> {
        let mut stream = UnixStream::connect(&self.ipc_path).map_err(|error| {
            SymphonyError::playback(format!("Could not connect to mpv: {error}"))
        })?;
        stream
            .set_write_timeout(Some(Duration::from_secs(1)))
            .map_err(|error| SymphonyError::playback(format!("Could not control mpv: {error}")))?;
        let mut payload = serde_json::to_vec(&command).map_err(|error| {
            SymphonyError::playback(format!("Could not encode mpv command: {error}"))
        })?;
        payload.push(b'\n');
        stream
            .write_all(&payload)
            .map_err(|error| SymphonyError::playback(format!("Could not control mpv: {error}")))
    }

    fn stop_process(&mut self) {
        if self.child.is_some() {
            let _ = self.send_command(serde_json::json!({ "command": ["quit"] }));
        }
        if let Some(mut child) = self.child.take() {
            let _ = child.kill();
            let _ = child.wait();
        }
        if self.ipc_path.exists() {
            let _ = std::fs::remove_file(&self.ipc_path);
        }
    }
}

impl Drop for MpvPlaybackEngine {
    fn drop(&mut self) {
        self.stop_process();
    }
}

#[async_trait]
impl PlaybackEngine for MpvPlaybackEngine {
    async fn play(&mut self, track_id: &TrackId, stream_url: &str) -> Result<()> {
        self.stop_process();
        let (media_url, duration, is_demo) = Self::media_url(stream_url)?;
        let ipc_option = format!("--input-ipc-server={}", self.ipc_path.display());
        let volume_option = format!(
            "--volume={}",
            if self.muted { 0.0 } else { self.volume * 100.0 }
        );
        let yt_dlp_option = format!(
            "--script-opts=ytdl_hook-ytdl_path={}",
            self.yt_dlp_path.display()
        );

        let mut command = Command::new("mpv");
        command
            .arg("--no-config")
            .arg("--no-video")
            .arg("--no-terminal")
            .arg("--really-quiet")
            .arg("--ao=pulse")
            .arg("--audio-display=no")
            .arg("--ytdl-format=bestaudio")
            .arg(yt_dlp_option)
            .arg(ipc_option)
            .arg(volume_option)
            .stdin(Stdio::null())
            .stdout(Stdio::null())
            .stderr(Stdio::null());
        if is_demo {
            command.arg("--length=240");
        }
        command.arg("--").arg(media_url);

        let mut child = command.spawn().map_err(|error| {
            SymphonyError::playback(format!(
                "Could not start mpv. Install mpv and yt-dlp first: {error}"
            ))
        })?;

        for _ in 0..100 {
            if self.ipc_path.exists() {
                break;
            }
            if let Some(exit) = child
                .try_wait()
                .map_err(|error| SymphonyError::playback(format!("Could not start mpv: {error}")))?
            {
                return Err(SymphonyError::playback(format!(
                    "mpv exited before playback started ({exit})"
                )));
            }
            tokio::time::sleep(Duration::from_millis(20)).await;
        }

        if !self.ipc_path.exists() {
            let _ = child.kill();
            let _ = child.wait();
            return Err(SymphonyError::playback(
                "mpv did not create its control socket",
            ));
        }

        self.child = Some(child);
        self.status = PlaybackStatus::Playing;
        self.paused_position = Duration::ZERO;
        self.duration = duration;
        self.current_track_id = Some(track_id.clone());
        self.play_start = Some(Instant::now());
        Ok(())
    }

    async fn pause(&mut self) -> Result<()> {
        if self.status != PlaybackStatus::Playing {
            return Err(SymphonyError::playback("Cannot pause: not playing"));
        }
        self.send_command(serde_json::json!({
            "command": ["set_property", "pause", true]
        }))?;
        self.paused_position = self.position();
        self.play_start = None;
        self.status = PlaybackStatus::Paused;
        Ok(())
    }

    async fn resume(&mut self) -> Result<()> {
        if self.status != PlaybackStatus::Paused {
            return Err(SymphonyError::playback("Cannot resume: not paused"));
        }
        self.send_command(serde_json::json!({
            "command": ["set_property", "pause", false]
        }))?;
        self.play_start = Some(Instant::now());
        self.status = PlaybackStatus::Playing;
        Ok(())
    }

    async fn stop(&mut self) -> Result<()> {
        self.stop_process();
        self.status = PlaybackStatus::Stopped;
        self.paused_position = Duration::ZERO;
        self.current_track_id = None;
        self.play_start = None;
        Ok(())
    }

    async fn seek(&mut self, position: Duration) -> Result<()> {
        if self.child.is_none() {
            return Err(SymphonyError::playback("Cannot seek: not playing"));
        }
        self.send_command(serde_json::json!({
            "command": ["seek", position.as_secs_f64(), "absolute+exact"]
        }))?;
        self.paused_position = position;
        if self.status == PlaybackStatus::Playing {
            self.play_start = Some(Instant::now());
        }
        Ok(())
    }

    fn set_volume(&mut self, volume: f64) {
        self.volume = volume.clamp(0.0, 1.0);
        if self.child.is_some() && !self.muted {
            let _ = self.send_command(serde_json::json!({
                "command": ["set_property", "volume", self.volume * 100.0]
            }));
        }
    }

    fn volume(&self) -> f64 {
        self.volume
    }

    fn status(&self) -> PlaybackStatus {
        if self.child.is_none()
            || (self.status == PlaybackStatus::Playing
                && !self.duration.is_zero()
                && self.position() >= self.duration)
        {
            PlaybackStatus::Stopped
        } else {
            self.status
        }
    }

    fn position(&self) -> Duration {
        let position = match self.status {
            PlaybackStatus::Playing => self.paused_position + self.elapsed(),
            PlaybackStatus::Paused => self.paused_position,
            PlaybackStatus::Stopped => Duration::ZERO,
        };
        if self.duration.is_zero() {
            position
        } else {
            position.min(self.duration)
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
        if self.child.is_some() {
            let _ = self.send_command(serde_json::json!({
                "command": ["set_property", "mute", muted]
            }));
        }
    }
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
    #[cfg(test)]
    return Box::new(MockPlaybackEngine::new());

    #[cfg(not(test))]
    Box::new(MpvPlaybackEngine::new())
}
