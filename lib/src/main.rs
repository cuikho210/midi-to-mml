extern crate revelation_mobile_midi_to_mml;

use revelation_mobile_midi_to_mml::{MmlSong, MmlSongOptions};

fn main() {
    let path = "/home/cuikho210/Projects/revelation-mobile-midi-to-mml/lib_player/test_resources/midi/Hitchcock.mid";
    let mut song = MmlSong::from_path(path, MmlSongOptions {
        auto_boot_velocity: true,
        auto_equalize_note_length: true,
        ..Default::default()
    }).unwrap();

    song.split_track(0).unwrap();

    for track in song.tracks.iter() {
        println!(
            "Track {} - {} - {} notes --------------------------",
            track.name,
            track.instrument.name,
            track.mml_note_length,
        );

        println!("{}\n", track.to_mml_debug());
    }
}
