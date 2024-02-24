use std::{
    fs::File,
    io::BufReader,
    sync::{Arc, Mutex},
    thread
};
use rodio::{Decoder, OutputStream, Sink};
use cpal::traits::HostTrait;

pub struct MusicPlayer {
    sink: Arc<Mutex<Sink>>,
    _stream: Arc<Mutex<OutputStream>>
}

impl MusicPlayer {
    pub fn new() -> Self {
        let host = cpal::default_host();
        let output_device = host.default_output_device().expect("No output device available");

        let (_stream, stream_handle) = OutputStream::try_from_device(&output_device).expect("Failed to create stream");
        let sink = Sink::try_new(&stream_handle).unwrap();

        MusicPlayer { 
            sink: Arc::new(Mutex::new(sink)),
            _stream: Arc::new(Mutex::new(_stream))
        }
    }

    pub fn play(&mut self, file_name: String) {
        let path_str = format!("assets/music/{}.mp3", file_name);
        
        let cloned_sink = Arc::clone(&self.sink);

        let _thread = thread::spawn(move || {
            let file = File::open(&path_str).unwrap();
            let reader = BufReader::new(file);
            let decoder = Decoder::new_looped(reader).expect("Failed to create decoder");
            
            let sink = cloned_sink.lock().unwrap();
            
            sink.stop();
            sink.append(decoder);
            sink.play();
        });
    }

    pub fn pause(&self) {
        let sink = self.sink.lock().unwrap();
        sink.pause()
    }

    pub fn stop(&self) {
        let sink = self.sink.lock().unwrap();
        sink.stop()
    }

    pub fn set_volume(&mut self, volume: f32) {
        let sink = self.sink.lock().unwrap();
        sink.set_volume(volume)
    }
}