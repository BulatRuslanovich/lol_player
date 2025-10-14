use rodio::{Decoder, OutputStream, OutputStreamBuilder, Sink};
use std::fs::File;
use std::io::BufReader;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;
use walkdir::WalkDir;

pub struct AudioPlayer {
    sink: Arc<Mutex<Sink>>,
    _stream: Arc<OutputStream>,
    songs: Arc<Mutex<Vec<(u32, PathBuf)>>>,
    current_index: Arc<Mutex<Option<usize>>>,
    is_playing: Arc<Mutex<bool>>,
}

impl AudioPlayer {
    pub fn new() -> Self {
        let stream =
            OutputStreamBuilder::open_default_stream().expect("Не удалось открыть аудио-поток");
        let sink = Sink::connect_new(&stream.mixer());

        let player = AudioPlayer {
            sink: Arc::new(Mutex::new(sink)),
            _stream: Arc::new(stream),
            songs: Arc::new(Mutex::new(Vec::new())),
            current_index: Arc::new(Mutex::new(None)),
            is_playing: Arc::new(Mutex::new(false)),
        };
        
        player
    }

    pub fn load_songs_from_dir(&self, dir: &str) {
        let mut songs = self.songs.lock().unwrap();
        let mut current_index = self.current_index.lock().unwrap();

        songs.clear();
        *current_index = None;

        let mut index = 0;
        for entry in WalkDir::new(dir)
            .follow_links(true)
            .into_iter()
            .filter_map(|e| e.ok())
        {
            let path = entry.path();
            if let Some(ext) = path.extension() {
                let ext = ext.to_string_lossy().to_lowercase();
                if matches!(ext.as_str(), "mp3" | "wav" | "flac" | "ogg") {
                    songs.push((index, path.to_path_buf()));
                    index += 1;
                }
            }
        }
        
        songs.sort_by(|a, b| a.1.file_name().cmp(&b.1.file_name()));
    }

    pub fn start_playback_monitor(self: Arc<Self>) {
        let player_clone = Arc::clone(&self);
        
        thread::spawn(move || {
            loop {
                thread::sleep(Duration::from_millis(500));
                
                let should_play_next = {
                    let sink = player_clone.sink.lock().unwrap();
                    let is_playing = player_clone.is_playing.lock().unwrap();
                    let songs = player_clone.songs.lock().unwrap();
                    
                    sink.empty() && *is_playing && !songs.is_empty()
                };
                
                if should_play_next {
                    player_clone.next();
                }
            }
        });
    }

    pub fn play_by_index(&self, index: usize) {
        let songs = self.songs.lock().unwrap();
        if let Some(song) = songs.get(index) {
            self.play_by_file(&song.1);
            *self.current_index.lock().unwrap() = Some(index);
            *self.is_playing.lock().unwrap() = true;
        }
    }

    pub fn play_by_file(&self, file_path: &PathBuf) {
        if let Ok(file) = File::open(file_path) {
            if let Ok(source) = Decoder::new(BufReader::new(file)) {
                let sink = self.sink.lock().unwrap();
                sink.stop();
                sink.append(source);
                sink.play();
            }
        }
    }

    pub fn toggle_play_pause(&self) {
        let sink = self.sink.lock().unwrap();
        let mut is_playing = self.is_playing.lock().unwrap();
        
        if sink.is_paused() {
            sink.play();
            *is_playing = true;
        } else {
            sink.pause();
            *is_playing = false;
        }
    }

    // pub fn pause(&self) {
    //     let sink = self.sink.lock().unwrap();
    //     sink.pause();
    //     *self.is_playing.lock().unwrap() = false;
    // }

    // pub fn play(&self) {
    //     let sink = self.sink.lock().unwrap();
    //     sink.play();
    //     *self.is_playing.lock().unwrap() = true;
    // }

    pub fn next(&self) {
        let current_index = *self.current_index.lock().unwrap();
        let songs = self.songs.lock().unwrap();
        
        if let Some(mut index) = current_index {
            index = (index + 1) % songs.len();
            drop(songs); // Освобождаем lock перед вызовом play_by_index
            
            self.play_by_index(index);
        } else if !songs.is_empty() {
            drop(songs);
            self.play_by_index(0);
        }
    }

    pub fn previous(&self) {
        let current_index = *self.current_index.lock().unwrap();
        let songs = self.songs.lock().unwrap();
        
        if let Some(mut index) = current_index {
            index = if index == 0 { songs.len() - 1 } else { index - 1 };
            drop(songs);
            
            self.play_by_index(index);
        } else if !songs.is_empty() {
            drop(songs);
            self.play_by_index(0);
        }
    }

    pub fn get_songs(&self) -> Vec<(u32, PathBuf)> {
        self.songs.lock().unwrap().clone()
    }

    // pub fn get_current_index(&self) -> Option<usize> {
    //     *self.current_index.lock().unwrap()
    // }

    pub fn is_playing(&self) -> bool {
        *self.is_playing.lock().unwrap()
    }

    pub fn get_current_song(&self) -> Option<PathBuf> {
        let current_index = *self.current_index.lock().unwrap();
        let songs = self.songs.lock().unwrap();
        
        current_index.and_then(|idx| songs.get(idx).map(|song| song.1.clone()))
    }
}

impl Default for AudioPlayer {
    fn default() -> Self {
        Self::new()
    }
}