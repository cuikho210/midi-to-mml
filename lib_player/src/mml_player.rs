use std::{
    path::PathBuf, sync::Arc,
    thread::{self, JoinHandle}, time::{Instant, Duration},
};
use cpal::Stream;
use revelation_mobile_midi_to_mml::{Instrument, Song};
use crate::{Parser, SynthOutputConnection, Synth};

pub struct MmlPlayerOptions {
    pub soundfont_path: PathBuf,
}

pub struct MmlPlayer {
    pub synth: Synth,
    pub stream: Stream,
    pub connection: SynthOutputConnection,
    pub tracks: Vec<Arc<Parser>>,
}

impl MmlPlayer {
    pub fn new(options: MmlPlayerOptions) -> Self {
        let time = Instant::now();

        let synth = Synth::new();
        let (stream, connection) = synth.new_stream(options.soundfont_path);

        log_initialize_synth(time.elapsed());
        
        Self {
            synth,
            stream,
            connection,
            tracks: Vec::new(),
        }
    }

    pub fn from_song(song: &Song, options: MmlPlayerOptions) -> Self {
        let mmls: Vec<(String, Instrument)> = song.tracks.iter().map::<(String, Instrument), _>(|track| {
            (track.to_mml(), track.instrument.to_owned())
        }).collect();

        Self::from_mmls(mmls, options)
    }

    pub fn from_mmls(mmls: Vec<(String, Instrument)>, options: MmlPlayerOptions) -> Self {
        let mut result = Self::new(options);
        result.parse_mmls(mmls);

        result
    }

    pub fn parse_mmls(&mut self, mmls: Vec<(String, Instrument)>) {
        let mut handles: Vec<JoinHandle<Parser>> = Vec::new();
        let mut tracks: Vec<Arc<Parser>> = Vec::new();

        let time = Instant::now();
        let track_length = mmls.len();
        let mut char_length: usize = 0;

        for mml in mmls {
            let conn = self.connection.clone();
            char_length += mml.0.len();

            let handle = thread::spawn::<_, Parser>(move || {
                Parser::parse(mml.0, mml.1, conn)
            });
            handles.push(handle);
        }

        for handle in handles {
            let parsed = handle.join().unwrap();
            tracks.push(Arc::new(parsed));
        }

        log_parse_mmls(time.elapsed(), track_length, char_length);
        self.tracks = tracks;
    }

    pub fn play(&self) {
        let mut handles: Vec<JoinHandle<()>> = Vec::new();

        for track in self.tracks.iter() {
            let parsed = track.clone();
            let handle = thread::spawn(move || parsed.play());
            handles.push(handle);
        }

        for handle in handles {
            handle.join().unwrap();
        }
    }
}

fn log_initialize_synth(duration: Duration) {
    println!("Initialized synth in {}ms", duration.as_millis());
}

fn log_parse_mmls(duration: Duration, track_length: usize, char_length: usize) {
    println!("Parsed {} tracks, {} chars in {}ms", track_length, char_length, duration.as_millis());
}
