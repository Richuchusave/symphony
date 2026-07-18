use rand::Rng;

use crate::types::*;

pub struct PlaybackQueue {
    pub tracks: Vec<TrackId>,
    pub current_index: Option<usize>,
    pub history: Vec<TrackId>,
    pub repeat: RepeatMode,
    pub shuffle: ShuffleMode,
}

impl PlaybackQueue {
    pub fn new() -> Self {
        Self {
            tracks: Vec::new(),
            current_index: None,
            history: Vec::new(),
            repeat: RepeatMode::Off,
            shuffle: ShuffleMode::Off,
        }
    }
}

impl Default for PlaybackQueue {
    fn default() -> Self {
        Self::new()
    }
}

impl PlaybackQueue {
    pub fn len(&self) -> usize {
        self.tracks.len()
    }

    pub fn is_empty(&self) -> bool {
        self.tracks.is_empty()
    }

    pub fn is_current(&self, index: usize) -> bool {
        self.current_index == Some(index)
    }

    pub fn add(&mut self, track_id: TrackId) {
        self.tracks.push(track_id);
    }

    pub fn remove(&mut self, index: usize) -> bool {
        if index >= self.tracks.len() {
            return false;
        }
        self.tracks.remove(index);
        if let Some(current) = self.current_index {
            if index < current {
                self.current_index = Some(current.saturating_sub(1));
            } else if index == current {
                if self.tracks.is_empty() {
                    self.current_index = None;
                } else if index >= self.tracks.len() {
                    self.current_index = Some(self.tracks.len().saturating_sub(1));
                }
            }
        }
        true
    }

    pub fn clear(&mut self) {
        self.tracks.clear();
        self.current_index = None;
        self.history.clear();
    }

    pub fn move_track(&mut self, from: usize, to: usize) {
        if from >= self.tracks.len() || to > self.tracks.len() {
            return;
        }
        let track = self.tracks.remove(from);
        self.tracks.insert(to, track);

        if let Some(current) = self.current_index {
            if from == current {
                self.current_index = Some(to);
            } else if from < current && to >= current {
                self.current_index = Some(current.saturating_sub(1));
            } else if from > current && to <= current {
                self.current_index = Some(current.saturating_add(1));
            }
        }
    }

    pub fn current(&self) -> Option<&TrackId> {
        self.current_index.and_then(|i| self.tracks.get(i))
    }

    #[allow(clippy::should_implement_trait)]
    pub fn next(&mut self) -> Option<TrackId> {
        match self.shuffle {
            ShuffleMode::On => self.next_shuffled(),
            ShuffleMode::Off => self.next_sequential(),
        }
    }

    fn next_sequential(&mut self) -> Option<TrackId> {
        match self.repeat {
            RepeatMode::One => self.current().cloned(),
            RepeatMode::All => {
                if self.tracks.is_empty() {
                    return None;
                }
                match self.current_index {
                    None => {
                        self.current_index = Some(0);
                        self.tracks.first().cloned()
                    }
                    Some(i) => {
                        let next = (i + 1) % self.tracks.len();
                        self.current_index = Some(next);
                        Some(self.tracks[next].clone())
                    }
                }
            }
            RepeatMode::Off => {
                match self.current_index {
                    None => {
                        if self.tracks.is_empty() {
                            return None;
                        }
                        self.current_index = Some(0);
                        self.tracks.first().cloned()
                    }
                    Some(i) => {
                        let next = i + 1;
                        if next >= self.tracks.len() {
                            self.current_index = None;
                            return None;
                        }
                        self.current_index = Some(next);
                        Some(self.tracks[next].clone())
                    }
                }
            }
        }
    }

    fn next_shuffled(&mut self) -> Option<TrackId> {
        if self.tracks.is_empty() {
            return None;
        }
        let indices = self.shuffle_indices();
        let current_pos = self.current_index.and_then(|i| indices.iter().position(|&x| x == i));

        match self.repeat {
            RepeatMode::One => self.current().cloned(),
            RepeatMode::All | RepeatMode::Off => {
                let next_pos = current_pos.map(|p| p + 1).unwrap_or(0);
                if next_pos >= indices.len() {
                    if self.repeat == RepeatMode::All {
                        self.current_index = Some(indices[0]);
                        Some(self.tracks[indices[0]].clone())
                    } else {
                        self.current_index = None;
                        None
                    }
                } else {
                    self.current_index = Some(indices[next_pos]);
                    Some(self.tracks[indices[next_pos]].clone())
                }
            }
        }
    }

    pub fn previous(&mut self) -> Option<TrackId> {
        if self.tracks.is_empty() {
            return None;
        }

        match self.current_index {
            None => {
                self.current_index = Some(0);
                self.tracks.first().cloned()
            }
            Some(i) => {
                if self.repeat == RepeatMode::Off && i == 0 {
                    self.current_index = None;
                    return None;
                }
                let prev = match self.repeat {
                    RepeatMode::Off => i.saturating_sub(1),
                    RepeatMode::All | RepeatMode::One => {
                        if i == 0 {
                            self.tracks.len() - 1
                        } else {
                            i - 1
                        }
                    }
                };
                self.current_index = Some(prev);
                Some(self.tracks[prev].clone())
            }
        }
    }

    pub fn play_index(&mut self, index: usize) -> Option<&TrackId> {
        if index >= self.tracks.len() {
            return None;
        }
        if let Some(current) = self.current_index {
            self.history.push(self.tracks[current].clone());
        }
        self.current_index = Some(index);
        self.tracks.get(index)
    }

    pub fn set_queue(&mut self, tracks: Vec<TrackId>, start_index: usize) {
        self.tracks = tracks;
        self.history.clear();
        if !self.tracks.is_empty() && start_index < self.tracks.len() {
            self.current_index = Some(start_index);
        } else {
            self.current_index = None;
        }
    }

    pub fn shuffle_indices(&self) -> Vec<usize> {
        let mut indices: Vec<usize> = (0..self.tracks.len()).collect();
        let mut rng = rand::thread_rng();
        let len = indices.len();
        for i in (1..len).rev() {
            let j = rng.gen_range(0..=i);
            indices.swap(i, j);
        }
        indices
    }

    pub fn toggle_repeat(&mut self) {
        self.repeat = self.repeat.cycle();
    }

    pub fn toggle_shuffle(&mut self) {
        self.shuffle = match self.shuffle {
            ShuffleMode::Off => ShuffleMode::On,
            ShuffleMode::On => ShuffleMode::Off,
        };
    }

    pub fn queue_from_current(&self) -> Vec<TrackId> {
        match self.current_index {
            None => self.tracks.clone(),
            Some(i) => {
                if i >= self.tracks.len() {
                    return Vec::new();
                }
                self.tracks[i..].to_vec()
            }
        }
    }
}
