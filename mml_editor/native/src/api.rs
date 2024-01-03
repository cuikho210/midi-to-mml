use midi_to_mml::{SongOptions, Song};
use midly::Smf;

pub fn parse_midi(bytes: Vec<u8>, is_auto_split: bool, to_merge: Vec<(usize, usize)>) -> Vec<String> {
    let mut result: Vec<String> = Vec::new();
    let song = Song::from_bytes(bytes, SongOptions {
        is_split_track: is_auto_split,
        merge_track: to_merge,
    }).unwrap();

    for track in song.tracks.iter() {
        result.push(track.to_mml());
    }

    result
}

pub fn get_track_length(bytes: Vec<u8>) -> usize {
    let smf = Smf::parse(&bytes).unwrap();
    smf.tracks.len()
}
