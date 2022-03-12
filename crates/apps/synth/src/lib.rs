use num::FromPrimitive;
use rimd::Status;

use audio_processor_traits::{
    AudioBuffer, AudioProcessor, AudioProcessorSettings, BufferProcessor, MidiEventHandler,
    MidiMessageLike, SimpleAudioProcessor,
};
use augmented_dsp_filters::rbj::{FilterProcessor, FilterType};
use voice::Voice;

mod voice;

pub struct Synthesizer {
    current_voice: usize,
    voices: [Voice; 4],
    filter: BufferProcessor<FilterProcessor<f32>>,
}

impl Default for Synthesizer {
    fn default() -> Self {
        let settings = AudioProcessorSettings::default();
        Self::new(settings.sample_rate)
    }
}

impl Synthesizer {
    pub fn new(sample_rate: f32) -> Self {
        Synthesizer {
            current_voice: 0,
            voices: [
                Voice::new(sample_rate),
                Voice::new(sample_rate),
                Voice::new(sample_rate),
                Voice::new(sample_rate),
            ],
            filter: BufferProcessor(FilterProcessor::new(FilterType::LowPass)),
        }
    }
}

impl AudioProcessor for Synthesizer {
    type SampleType = f32;

    fn prepare(&mut self, settings: AudioProcessorSettings) {
        for voice in &mut self.voices {
            voice.prepare(settings);
        }
        self.filter.prepare(settings);
    }

    fn process<BufferType: AudioBuffer<SampleType = Self::SampleType>>(
        &mut self,
        data: &mut BufferType,
    ) {
        // Silence the input
        for sample in data.slice_mut() {
            *sample = 0.0;
        }

        // Produce output for 4 voices
        for voice in &mut self.voices {
            voice.process(data);
        }

        self.filter.process(data);
    }
}

impl MidiEventHandler for Synthesizer {
    fn process_midi_events<Message: MidiMessageLike>(&mut self, midi_messages: &[Message]) {
        for message in midi_messages {
            let maybe_bytes = message.bytes();
            // TODO Write a better MIDI parser
            // (rimd allocates on parse, so we can't parse the whole message; only the status byte)
            let maybe_status = maybe_bytes.map(|b| rimd::Status::from_u8(b[0])).flatten();
            if let Some((status, bytes)) = maybe_status.zip(maybe_bytes) {
                self.handle_midi_message(status, bytes);
            }
        }
    }
}

impl Synthesizer {
    fn handle_midi_message(&mut self, status: rimd::Status, bytes: &[u8]) {
        match status {
            Status::NoteOn => {
                println!("Note on {}/{}", bytes[1], bytes[2]);
                self.note(false, bytes);
            }
            Status::NoteOff => {
                // println!("Note off {}", bytes[1]);
                self.note(true, bytes);
            }
            Status::ControlChange => {
                if bytes[1] == 21 {
                    self.filter
                        .0
                        .set_cutoff(22000.0 * (bytes[2] as f32 / 127.0));
                }
                if bytes[1] == 22 {
                    self.filter.0.set_q(1.0 + (bytes[2] as f32 / 127.0));
                }
            }
            _ => {}
        }
    }

    fn note(&mut self, is_off: bool, bytes: &[u8]) {
        let note = bytes[1];
        let velocity = bytes[2];
        if velocity == 0 || is_off {
            let voice = self.voices.iter_mut().find(|voice| {
                voice.current_note().is_some() && voice.current_note().unwrap() == note
            });
            if let Some(voice) = voice {
                voice.note_off();
            }
        } else {
            let voice = self
                .voices
                .iter_mut()
                .find(|voice| voice.current_note().is_none());
            if let Some(voice) = voice {
                voice.note_on(note, velocity);
            } else {
                self.current_voice = (self.current_voice + 1) % self.voices.len();
                self.voices[self.current_voice].note_on(note, velocity);
            }
        }
    }
}

#[cfg(test)]
mod test {
    #[test]
    fn test_compiles() {}
}
