use std::sync::atomic::{AtomicBool, Ordering};
use std::time::Duration;

use adsr_envelope::Envelope;
use audio_garbage_collector::{make_shared, Shared};
use audio_processor_traits::{AtomicF32, AudioBuffer, AudioProcessor, AudioProcessorSettings};
use augmented_oscillator::Oscillator;
use augmented_playhead::{PlayHead, PlayHeadOptions};
pub use bridge_generated::*;

mod api;
mod bridge_generated;

pub struct MetronomeProcessorHandle {
    is_playing: AtomicBool,
    tempo: AtomicF32,
    volume: AtomicF32,
}

struct MetronomeProcessorState {
    playhead: PlayHead,
    oscillator: Oscillator<f32>,
    playing: bool,
    envelope: Envelope,
}

pub struct MetronomeProcessor {
    state: MetronomeProcessorState,
    handle: Shared<MetronomeProcessorHandle>,
}

impl Default for MetronomeProcessor {
    fn default() -> Self {
        Self::new()
    }
}

impl MetronomeProcessor {
    pub fn new() -> Self {
        let mut envelope = Envelope::new();
        envelope.set_attack(Duration::from_millis(2));
        envelope.set_decay(Duration::from_millis(50));
        envelope.set_sustain(0.8);
        envelope.set_release(Duration::from_millis(10));

        MetronomeProcessor {
            handle: make_shared(MetronomeProcessorHandle {
                is_playing: AtomicBool::new(true),
                tempo: AtomicF32::new(120.0),
                volume: AtomicF32::new(0.3),
            }),
            state: MetronomeProcessorState {
                playhead: PlayHead::new(PlayHeadOptions {
                    sample_rate: Some(44100.0),
                    ticks_per_quarter_note: Some(16),
                    tempo: Some(120),
                }),
                oscillator: Oscillator::sine(44100.0),
                playing: false,
                envelope,
            },
        }
    }
}

impl AudioProcessor for MetronomeProcessor {
    type SampleType = f32;

    fn prepare(&mut self, settings: AudioProcessorSettings) {
        self.state.playhead = PlayHead::new(PlayHeadOptions {
            sample_rate: Some(settings.sample_rate()),
            tempo: Some(self.handle.tempo.get() as u32),
            ..*self.state.playhead.options()
        });
        self.state.envelope.set_sample_rate(settings.sample_rate());
        self.state
            .oscillator
            .set_sample_rate(settings.sample_rate())
    }

    fn process<BufferType: AudioBuffer<SampleType = Self::SampleType>>(
        &mut self,
        data: &mut BufferType,
    ) {
        if !self.handle.is_playing.load(Ordering::Relaxed) {
            for sample in data.slice_mut() {
                *sample = 0.0;
            }
            return;
        }

        let tempo = self.handle.tempo.get() as u32;
        if Some(tempo) != self.state.playhead.options().tempo {
            self.state.playhead.set_tempo(tempo);
        }

        for frame in data.frames_mut() {
            self.state.playhead.accept_samples(1);
            self.state.envelope.tick();
            self.state.oscillator.tick();

            let position = self.state.playhead.position_beats();
            if position % 4.0 < 0.1 {
                self.state.oscillator.set_frequency(880.0);
            } else {
                self.state.oscillator.set_frequency(440.0);
            }

            if position % 1.0 < 0.1 && !self.state.playing {
                self.state.playing = true;
                self.state.envelope.note_on();
            } else {
                self.state.playing = false;
                self.state.envelope.note_off();
            }

            let out = self.state.oscillator.get()
                * self.handle.volume.get()
                * self.state.envelope.volume();

            for sample in frame {
                *sample = out;
            }
        }
    }
}
