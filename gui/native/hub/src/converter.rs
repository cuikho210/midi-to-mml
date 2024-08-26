use crate::messages::types::{SignalInstrument, SignalMmlSongOptions, SignalMmlTrack};
use revelation_mobile_midi_to_mml::{instrument::Instrument, mml_track::MmlTrack, MmlSongOptions};

pub fn mml_song_options_to_signal(options: &MmlSongOptions) -> SignalMmlSongOptions {
    SignalMmlSongOptions {
        auto_boot_velocity: options.auto_boot_velocity,
        auto_equalize_note_length: options.auto_equalize_note_length,
        velocity_min: options.velocity_min as u32,
        velocity_max: options.velocity_max as u32,
        min_gap_for_chord: options.min_gap_for_chord as u32,
        smallest_unit: options.smallest_unit as u32,
    }
}

pub fn instrument_to_signal(instrument: &Instrument) -> SignalInstrument {
    SignalInstrument {
        name: instrument.name.to_owned(),
        instrument_id: instrument.instrument_id as u32,
        midi_channel: instrument.midi_channel as u32,
    }
}

pub fn mml_song_tracks_to_signal(tracks: &Vec<MmlTrack>) -> Vec<SignalMmlTrack> {
    let mut list_signal_track: Vec<SignalMmlTrack> = Vec::new();

    for (index, track) in tracks.iter().enumerate() {
        let instrument = instrument_to_signal(&track.instrument);

        list_signal_track.push(SignalMmlTrack {
            index: index as u32,
            name: track.name.to_owned(),
            instrument: Some(instrument),
            mml: track.to_mml(),
            mml_note_length: track.mml_note_length as u64,
        });
    }

    list_signal_track
}

pub fn signal_to_mml_song_options(options: &SignalMmlSongOptions) -> MmlSongOptions {
    MmlSongOptions {
        auto_boot_velocity: options.auto_boot_velocity,
        auto_equalize_note_length: options.auto_equalize_note_length,
        merge_meta_tracks: false,
        velocity_min: options.velocity_min as u8,
        velocity_max: options.velocity_max as u8,
        min_gap_for_chord: options.min_gap_for_chord as u8,
        smallest_unit: options.smallest_unit as usize,
    }
}
