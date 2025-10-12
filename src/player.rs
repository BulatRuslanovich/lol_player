use rodio::{Decoder, OutputStream, OutputStreamBuilder, Sink};
use std::fs::File;
use std::io::BufReader;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use walkdir::WalkDir;

pub struct AudioPlayer {
    sink: Arc<Mutex<Sink>>,
    _stream: Arc<OutputStream>,
    current_file: Arc<Mutex<Option<PathBuf>>>,
    songs: Arc<Mutex<Vec<PathBuf>>>,
}

impl AudioPlayer {
    /// Создаёт новый аудиоплеер
    pub fn new() -> Self {
        let stream =
            OutputStreamBuilder::open_default_stream()
                .expect("Не удалось открыть аудио-поток");
        let sink = Sink::connect_new(&stream.mixer());

        AudioPlayer {
            sink: Arc::new(Mutex::new(sink)),
            _stream: Arc::new(stream),
            current_file: Arc::new(Mutex::new(None)),
            songs: Arc::new(Mutex::new(Vec::new())),
        }
    }

    /// Загружает все аудиофайлы из указанной директории
    pub fn load_songs_from_dir(&self, dir: &str) {
        let mut songs = self.songs.lock().unwrap();
        songs.clear();

        for entry in WalkDir::new(dir)
            .follow_links(true)
            .into_iter()
            .filter_map(|e| e.ok())
        {
            let path = entry.path();
            if let Some(ext) = path.extension() {
                let ext = ext.to_string_lossy().to_lowercase();
                if ext == "mp3" || ext == "wav" || ext == "flac" || ext == "ogg" {
                    songs.push(path.to_path_buf());
                }
            }
        }
    }

    /// Воспроизводит указанный файл
    pub fn play(&self, file_path: &PathBuf) {
        if let Ok(file) = File::open(file_path) {
            if let Ok(source) = Decoder::new(BufReader::new(file)) {
                let sink = self.sink.lock().unwrap();
                sink.stop();
                sink.append(source);
                sink.play();

                *self.current_file.lock().unwrap() = Some(file_path.clone());
            }
        }
    }

    /// Ставит на паузу или возобновляет воспроизведение
    pub fn pause(&self) {
        let sink = self.sink.lock().unwrap();
        if sink.is_paused() {
            sink.play();
        } else {
            sink.pause();
        }
    }

    /// Останавливает воспроизведение
    pub fn stop(&self) {
        let sink = self.sink.lock().unwrap();
        sink.stop();
    }

    /// Возвращает список всех загруженных песен
    pub fn get_songs(&self) -> Vec<PathBuf> {
        self.songs.lock().unwrap().clone()
    }

    /// Возвращает текущий воспроизводимый файл
    pub fn get_current_file(&self) -> Option<PathBuf> {
        self.current_file.lock().unwrap().clone()
    }
}