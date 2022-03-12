use std::sync::atomic::Ordering;

use audio_garbage_collector::{make_shared, Shared};
use audio_processor_graph::AudioProcessorGraph;
use audio_processor_traits::{
    AudioBuffer, AudioProcessor, AudioProcessorSettings, MidiEventHandler, MidiMessageLike,
    VecAudioBuffer,
};

use crate::{
    LoopSequencerProcessorHandle, LooperOptions, LooperProcessor, LooperProcessorHandle,
    TimeInfoProviderImpl,
};

pub struct LooperId(pub usize);

pub struct LooperVoice {
    looper_handle: Shared<LooperProcessorHandle>,
    sequencer_handle: Shared<LoopSequencerProcessorHandle>,
}

impl LooperVoice {
    pub fn looper(&self) -> &Shared<LooperProcessorHandle> {
        &self.looper_handle
    }

    pub fn sequencer(&self) -> &Shared<LoopSequencerProcessorHandle> {
        &self.sequencer_handle
    }
}

pub struct MultiTrackLooperHandle {
    voices: Vec<LooperVoice>,
    time_info_provider: Shared<TimeInfoProviderImpl>,
}

impl MultiTrackLooperHandle {
    pub fn start_recording(&self, looper_id: LooperId) {
        if let Some(handle) = self.voices.get(looper_id.0) {
            handle.looper_handle.start_recording();
        }
    }

    pub fn toggle_playback(&self, looper_id: LooperId) {
        if let Some(handle) = self.voices.get(looper_id.0) {
            handle.looper_handle.toggle_playback();
        }
    }

    pub fn clear(&self, looper_id: LooperId) {
        if let Some(handle) = self.voices.get(looper_id.0) {
            handle.looper_handle.clear();
        }
    }

    pub fn voices(&self) -> &Vec<LooperVoice> {
        &self.voices
    }

    pub fn get(&self, looper_id: LooperId) -> Option<&LooperVoice> {
        self.voices.get(looper_id.0)
    }

    pub fn time_info_provider(&self) -> &Shared<TimeInfoProviderImpl> {
        &self.time_info_provider
    }
}

pub struct MultiTrackLooper {
    graph: AudioProcessorGraph<VecAudioBuffer<f32>>,
    handle: Shared<MultiTrackLooperHandle>,
}

impl MultiTrackLooper {
    pub fn new(options: LooperOptions, num_voices: usize) -> Self {
        let time_info_provider = make_shared(TimeInfoProviderImpl::new(options.host_callback));
        let voices: Vec<LooperProcessor> = (0..num_voices)
            .map(|_| {
                let voice = LooperProcessor::new(options.clone(), time_info_provider.clone());
                voice.handle().tick_time.store(false, Ordering::Relaxed);
                voice
            })
            .collect();
        let handle = make_shared(MultiTrackLooperHandle {
            voices: voices
                .iter()
                .map(|voice| {
                    let looper_handle = voice.handle().clone();
                    let sequencer_handle = voice.sequencer_handle().clone();
                    LooperVoice {
                        looper_handle,
                        sequencer_handle,
                    }
                })
                .collect(),
            time_info_provider,
        });

        let mut graph = AudioProcessorGraph::default();
        let input_node = graph.input();
        let output_node = graph.output();
        for voice in voices {
            let voice_idx = graph.add_node(Box::new(voice));
            graph.add_connection(input_node, voice_idx);
            graph.add_connection(voice_idx, output_node);
        }

        Self { graph, handle }
    }

    pub fn handle(&self) -> &Shared<MultiTrackLooperHandle> {
        &self.handle
    }
}

impl AudioProcessor for MultiTrackLooper {
    type SampleType = f32;

    fn prepare(&mut self, settings: AudioProcessorSettings) {
        self.graph.prepare(settings);
    }

    fn process<BufferType: AudioBuffer<SampleType = Self::SampleType>>(
        &mut self,
        data: &mut BufferType,
    ) {
        self.graph.process(data);
    }
}

impl MidiEventHandler for MultiTrackLooper {
    fn process_midi_events<Message: MidiMessageLike>(&mut self, _midi_messages: &[Message]) {}
}
