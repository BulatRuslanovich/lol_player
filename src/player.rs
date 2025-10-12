use rodio::{Decoder, OutputStream, OutputStreamBuilder, Sink};
use std::fs::File;
use std::io::BufReader;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use walkdir::WalkDir;

pub struct AudioPlayer {
    sink: Arc<Mutex<Sink>>,
    _stream: Arc<OutputStream>,
    songs: Arc<Mutex<Vec<(u32, PathBuf)>>>,
    current_index: Arc<Mutex<u32>>,
}

impl AudioPlayer {
    pub fn new() -> Self {
        let stream =
            OutputStreamBuilder::open_default_stream().expect("Не удалось открыть аудио-поток");
        let sink = Sink::connect_new(&stream.mixer());

        AudioPlayer {
            sink: Arc::new(Mutex::new(sink)),
            _stream: Arc::new(stream),
            songs: Arc::new(Mutex::new(Vec::new())),
            current_index: Arc::new(Mutex::new(0)),
        }
    }

    pub fn load_songs_from_dir(&self, dir: &str) {
        let mut songs = self.songs.lock().unwrap();

        songs.clear();
        let mut index = 0;

        for entry in WalkDir::new(dir)
            .follow_links(true)
            .into_iter()
            .filter_map(|e| e.ok())
        {
            let path = entry.path();
            if let Some(ext) = path.extension() {
                let ext = ext.to_string_lossy().to_lowercase();
                if ext == "mp3" {
                    songs.push((index, path.to_path_buf()));
                    index += 1;
                }
            }
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

    pub fn pause(&self) {
        let sink = self.sink.lock().unwrap();
        if sink.is_paused() {
            sink.play();
        } else {
            sink.pause();
        }
    }

    pub fn get_songs(&self) -> Vec<(u32, PathBuf)> {
        self.songs.lock().unwrap().clone()
    }
}
