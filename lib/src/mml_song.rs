use std::{fmt::Debug, fs, path::Path, thread::{self, JoinHandle}};
use midly::{Smf, Timing, TrackEvent};
use crate::{mml_event::BridgeEvent, mml_track::MmlTrack, parser::{bridge_meta_from_midi_track, bridge_notes_from_midi_track}, utils};

#[derive(Debug, Clone)]
pub struct MmlSongOptions {
    pub auto_boot_velocity: bool,
    pub auto_equalize_note_length: bool,
    pub merge_meta_tracks: bool,
    pub velocity_min: u8,
    pub velocity_max: u8,
    pub min_gap_for_chord: u8,
    pub smallest_unit: usize,
}

impl Default for MmlSongOptions {
    fn default() -> Self {
        Self {
            auto_boot_velocity: false,
            auto_equalize_note_length: false,
            merge_meta_tracks: false,
            velocity_min: 0,
            velocity_max: 15,
            min_gap_for_chord: 0,
            smallest_unit: 64,
        }
    }
}

#[derive(Debug, Clone)]
pub struct MmlSong {
    pub ppq: u16,
    pub tracks: Vec<MmlTrack>,
    pub options: MmlSongOptions,
    velocity_diff: Option<u8>,
}

impl MmlSong {
    pub fn from_path<P>(path: P, options: MmlSongOptions) -> Result<Self, String>
    where
        P: AsRef<Path>,
    {
        let bytes = match fs::read(path) {
            Ok(bytes) => bytes,
            Err(err) => return Err(err.to_string()),
        };

        Self::from_bytes(bytes, options)
    }

    pub fn from_bytes(bytes: Vec<u8>, options: MmlSongOptions) -> Result<Self, String> {
        let smf = match Smf::parse(&bytes) {
            Ok(smf) => smf,
            Err(err) => return Err(err.to_string()),
        };

        let ppq = match get_ppq_from_smf(&smf) {
            Some(ppq) => ppq,
            None => 480,
        };

        let meta_events = get_bridge_meta_events(&smf.tracks, &options);

        let smf_tracks = smf.make_static().tracks;
        let bridge_note_events = get_bridge_note_events(smf_tracks);

        let tracks = bridge_events_to_tracks(meta_events, bridge_note_events, &options, ppq);

        let mut song = Self {
            ppq,
            tracks,
            options,
            velocity_diff: None,
        };
        song.appy_song_options();

        Ok(song)
    }

    pub fn merge_tracks(&mut self, index_a: usize, index_b: usize) -> Result<(), String> {
        let mut track_b = self.tracks.get(index_b).ok_or(
            format!("Cannot get track by index_b = {}", index_b)
        )?.to_owned();

        let track_a = self.tracks.get_mut(index_a).ok_or(
            format!("Cannot get track by index_a = {}", index_a)
        )?;

        track_a.merge(&mut track_b);
        self.tracks.remove(index_b);

        Ok(())
    }

    pub fn split_track(&mut self, index: usize) -> Result<(), String> {
        let track = self.tracks.get_mut(index).ok_or(
            format!("Cannot get track by index {}", index)
        )?;
        let (mut track_a, mut track_b) = track.split();

        if self.options.auto_boot_velocity {
            if let Some(velocity_diff) = self.velocity_diff {
                track_a.apply_boot_velocity(velocity_diff);
                track_b.apply_boot_velocity(velocity_diff);
            }
        }

        *track = track_a;
        self.tracks.insert(index + 1, track_b);

        Ok(())
    }

    pub fn set_song_options(&mut self, options: MmlSongOptions) -> Result<(), String> {
        self.options = options.clone();

        let mut handles: Vec<JoinHandle<MmlTrack>> = Vec::new();

        for track in self.tracks.iter() {
            let mut track = track.clone();
            track.song_options = options.clone();

            let handle = thread::spawn(move || {
                track.generate_mml_events();
                track
            });
            handles.push(handle);
        }

        self.tracks.clear();
        for handle in handles {
            let track = handle.join().ok().ok_or(
                String::from("[set_song_options] Cannot join thread")
            )?;
            self.tracks.push(track);
        }

        self.appy_song_options();
        Ok(())
    }

    fn appy_song_options(&mut self) {
        if self.options.auto_boot_velocity {
            let velocity_diff = utils::get_song_velocity_diff(&self.options, &self.tracks);
            self.velocity_diff = Some(velocity_diff);
            utils::auto_boot_song_velocity(&mut self.tracks, velocity_diff);
        }
    }
}

fn bridge_events_to_tracks(
    bridge_meta_events: Vec<Vec<BridgeEvent>>,
    bridge_events: Vec<Vec<BridgeEvent>>,
    song_options: &MmlSongOptions,
    ppq: u16,
) -> Vec<MmlTrack> {
    let mut tracks: Vec<MmlTrack> = Vec::new();
    let mut handles: Vec<JoinHandle<MmlTrack>> = Vec::new();

    for events in bridge_events {
        let options = song_options.to_owned();
        let index = handles.len();

        let meta_events = match song_options.merge_meta_tracks {
            true => bridge_meta_events.to_owned().into_iter().flatten().collect(),
            false => bridge_meta_events.get(handles.len()).unwrap().to_owned(),
        };

        let handle = thread::spawn::<_, MmlTrack>(move || {
            MmlTrack::from_bridge_events(index.to_string(), meta_events, events, options, ppq)
        });
        handles.push(handle);
    }

    for handle in handles {
        match handle.join() {
            Ok(track) => {
                tracks.push(track);
            }
            Err(_) => {
                eprintln!("[bridge_events_to_tracks] Cannot join thread");
            }
        }
    }

    tracks
}

fn get_bridge_note_events(smf_tracks: Vec<Vec<TrackEvent<'static>>>) -> Vec<Vec<BridgeEvent>> {
    let mut events: Vec<Vec<BridgeEvent>> = Vec::new();
    let mut handles: Vec<JoinHandle<Vec<BridgeEvent>>> = Vec::new();

    for track in smf_tracks {
        let handle = thread::spawn::<_, Vec<BridgeEvent>>(move || {
            bridge_notes_from_midi_track(&track)
        });
        handles.push(handle);
    }
    
    for handle in handles {
        match handle.join() {
            Ok(note_events) => {
                events.push(note_events);
            }
            Err(_) => {
                eprintln!("[bridge_smf_tracks] Cannot join thread");
            }
        }
    }

    events
}

fn get_bridge_meta_events(smf_tracks: &Vec<Vec<TrackEvent>>, song_options: &MmlSongOptions) -> Vec<Vec<BridgeEvent>> {
    let mut first_tempo: Option<BridgeEvent> = None;
    let mut meta_events: Vec<Vec<BridgeEvent>> = Vec::new();

    for track in smf_tracks.iter() {
        let mut events = bridge_meta_from_midi_track(track);

        if song_options.merge_meta_tracks == false {
            if let Some(tempo_event) = first_tempo.as_ref() {
                events.insert(0, tempo_event.to_owned());
                println!("Inserted a tempo event info events");
            } else {
                for event in events.iter() {
                    if let BridgeEvent::Tempo(_tempo, state) = event {
                        if state.position_in_tick == 0 {
                            first_tempo = Some(event.to_owned());
                        }
                    }
                }
            }
        }

        meta_events.push(events);
    }

    meta_events
}

fn get_ppq_from_smf(smf: &Smf) -> Option<u16> {
    match smf.header.timing {
        Timing::Metrical(ppq) => Some(ppq.as_int()),
        _ => None,
    }
}

