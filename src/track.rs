use std::collections::HashMap;
use midly::{TrackEventKind, MetaMessage, MidiMessage};
use crate::{
    track_event::TrackEvent,
    note::Note,
};

#[derive(Debug, Clone)]
pub struct Track {
    events: Vec<TrackEvent>,
}

impl Track {
    pub fn new(smf_track: &midly::Track, ppq: &u16) -> Self {
        let mut events: Vec<TrackEvent> = Vec::new();
        let mut holding_notes: HashMap<u8, usize> = HashMap::new();
        let mut current_tick: u32 = 0;

        for smf_event in smf_track.iter() {
            let delta = smf_event.delta.as_int();
            current_tick += delta;

            match smf_event.kind {
                TrackEventKind::Meta(message) => {
                    Self::match_meta_event(&message, &mut events);
                }
                TrackEventKind::Midi { message , .. } => {
                    Self::match_midi_event(
                        &message,
                        &mut events,
                        &mut holding_notes,
                        &current_tick,
                        ppq,
                    );
                }
                _ => ()
            }
        }

        Self {
            events,
        }
    }

    pub fn to_mml(&self) -> String {
        let mut result: Vec<String> = Vec::new();

        for event in self.events.iter() {
            result.push(event.to_mml());
        }

        result.join("")
    }

    fn match_meta_event(message: &MetaMessage, events: &mut Vec<TrackEvent>) {
        match message {
            MetaMessage::Tempo(tempo) => {
                events.push(TrackEvent::SetTempo(
                    60_000_000 / tempo.as_int()
                ));
            }
            _ => ()
        }
    }

    fn match_midi_event(
        message: &MidiMessage,
        events: &mut Vec<TrackEvent>,
        holding_notes: &mut HashMap<u8, usize>,
        current_ticks: &u32,
        ppq: &u16,
    ) {
        match message {
            MidiMessage::NoteOn { key, vel } => {
                let midi_key = key.as_int();

                if vel.as_int() == 0 {
                    Self::update_note(
                        midi_key,
                        events,
                        holding_notes,
                        current_ticks,
                        ppq,
                    );
                } else {
                    Self::create_note(
                        midi_key,
                        events,
                        holding_notes,
                        current_ticks,
                        ppq,
                    );
                }
            }
            MidiMessage::NoteOff { key, .. } => {
                let midi_key = key.as_int();

                Self::update_note(
                    midi_key,
                    events,
                    holding_notes,
                    current_ticks,
                    ppq,
                );
            }
            _ => ()
        }
    }

    fn create_note(
        midi_key: u8,
        events: &mut Vec<TrackEvent>,
        holding_notes: &mut HashMap<u8, usize>,
        current_ticks: &u32,
        ppq: &u16,
    ) {
        let note  = Note::new(
            midi_key,
            current_ticks.to_owned(),
            ppq,
        );

        if let Some(before_note) = Self::get_before_note(events) {
            let position_diff: i16 =
                note.position_in_note_64 as i16 - 
                (before_note.position_in_note_64 + before_note.duration_in_note_64) as i16
            ;

            // Rest
            if position_diff > 0 {
                events.push(TrackEvent::SetRest(
                    Note::get_duration(position_diff as u16)
                ));
            } else if position_diff == 0 {
                events.push(TrackEvent::ConnectChord);
            } else {
                events.push(TrackEvent::ConnectNote);
            }

            // Octave
            let octave_diff = note.octave as i8 - before_note.octave as i8;

            if octave_diff == 1 {
                events.push(TrackEvent::IncreOctave);
            } else if octave_diff == -1 {
                events.push(TrackEvent::DecreOctave);
            } else {
                events.push(TrackEvent::SetOctave(note.octave));
            }
        } else {
            let position_diff: i16 = note.position_in_note_64 as i16;

            // Rest
            if position_diff > 0 {
                events.push(TrackEvent::SetRest(
                    Note::get_duration(position_diff as u16)
                ));
            }

            events.push(TrackEvent::SetOctave(note.octave));
        }

        // Set note
        holding_notes.insert(midi_key, events.len());
        events.push(TrackEvent::SetNote(note));
    }

    fn update_note(
        midi_key: u8,
        events: &mut Vec<TrackEvent>,
        holding_notes: &HashMap<u8, usize>,
        current_ticks: &u32,
        ppq: &u16,
    ) {
        let index = holding_notes.get(&midi_key);
        if let Some(index) = index {
            if let Some(event) = events.get_mut(index.to_owned()) {
                if let TrackEvent::SetNote(note) = event {
                    let duration_in_ticks = current_ticks - note.position_in_tick;
                    let duration_in_note_64 = Note::tick_to_note_64(duration_in_ticks, ppq.to_owned());
                    let duration = Note::get_duration(duration_in_note_64);

                    note.duration_in_tick = duration_in_ticks;
                    note.duration_in_note_64 = duration_in_note_64;
                    note.duration = duration;
                }
            }
        }
    }

    fn get_before_note(events: &Vec<TrackEvent>) -> Option<Note> {
        if events.len() == 0 {
            return None;
        }

        for event in events.iter().rev() {
            match event {
                TrackEvent::SetNote(note) => return Some(note.to_owned()),
                _ => ()
            }
        }

        None
    }
}