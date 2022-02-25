use std::cmp::Ordering;
use std::time::Duration;

use iced::canvas::event::Status;
use iced::canvas::Cursor;
use iced::canvas::Event;
use iced::canvas::Fill;
use iced::canvas::Frame;
use iced::canvas::Geometry;
use iced::canvas::Program;
use iced::canvas::Stroke;
use iced::keyboard::KeyCode;
use iced::mouse;
use iced::mouse::ScrollDelta;
use iced::Canvas;
use iced::Column;
use iced::Container;
use iced::Element;
use iced::Length;
use iced::Point;
use iced::Rectangle;
use iced::Text;
use iced::{keyboard, Vector};

use audio_processor_iced_design_system::colors::Colors;
use audio_processor_traits::{AudioProcessor, InterleavedAudioBuffer, SimpleAudioProcessor};

use crate::ui::style::ContainerStyle;

pub struct AudioFileModel {
    samples: Vec<f32>,
    rms: Vec<f32>,
}

impl AudioFileModel {
    fn from_buffer(samples: Vec<f32>) -> Self {
        let max_sample = samples
            .iter()
            .cloned()
            .max_by(|f1, f2| f1.partial_cmp(f2).unwrap_or(Ordering::Equal))
            .unwrap_or(1.0);
        let samples: Vec<f32> = samples.iter().map(|f| f / max_sample).collect();
        let mut rms_processor =
            audio_processor_analysis::running_rms_processor::RunningRMSProcessor::new_with_duration(
                audio_garbage_collector::handle(),
                Duration::from_millis(30),
            );

        let mut rms_samples = vec![];
        rms_processor.prepare(Default::default());
        for sample in samples.iter() {
            rms_processor.s_process_frame(&mut [*sample]);
            rms_samples.push(rms_processor.handle().calculate_rms(0));
        }

        Self {
            samples,
            rms: rms_samples,
        }
    }

    fn samples_len(&self) -> usize {
        self.samples.len()
    }

    fn samples(&self) -> impl Iterator<Item = &f32> {
        self.samples.iter()
    }

    fn rms_len(&self) -> usize {
        self.rms.len()
    }

    fn rms(&self) -> impl Iterator<Item = &f32> {
        self.rms.iter()
    }
}

enum ChartMode {
    Samples,
    RMS,
}

struct VisualizationModel {
    zoom_x: f32,
    zoom_y: f32,
    offset: f32,
    chart_mode: ChartMode,
}

impl Default for VisualizationModel {
    fn default() -> Self {
        Self {
            zoom_x: 1.0,
            zoom_y: 1.0,
            offset: 0.0,
            chart_mode: ChartMode::Samples,
        }
    }
}

#[derive(Debug, Clone)]
pub enum Message {}

#[derive(Default)]
pub struct AudioEditorView {
    audio_file_model: Option<AudioFileModel>,
    visualization_model: VisualizationModel,
    shift_down: bool,
}

impl AudioEditorView {
    pub fn update(&mut self, _message: Message) {}

    pub fn view(&mut self) -> Element<Message> {
        // let empty_state = Text::new("Drop a file").into();
        Container::new(Canvas::new(self).width(Length::Fill).height(Length::Fill))
            .center_x()
            .center_y()
            .style(ContainerStyle)
            .width(Length::Fill)
            .height(Length::Fill)
            .into()
    }
}

impl Program<Message> for AudioEditorView {
    fn update(
        &mut self,
        event: Event,
        bounds: Rectangle,
        _cursor: Cursor,
    ) -> (Status, Option<Message>) {
        match event {
            Event::Mouse(mouse::Event::WheelScrolled {
                delta: ScrollDelta::Pixels { x, y },
            }) => {
                if self.shift_down {
                    self.visualization_model.zoom_x += x;
                    self.visualization_model.zoom_x =
                        self.visualization_model.zoom_x.min(100.0).max(1.0);
                    self.visualization_model.zoom_y += y;
                    self.visualization_model.zoom_y =
                        self.visualization_model.zoom_y.min(2.0).max(1.0);
                    (Status::Captured, None)
                } else {
                    let size = bounds.size();
                    let width = size.width * self.visualization_model.zoom_x;
                    let offset = (self.visualization_model.offset + x)
                        .max(0.0)
                        .min(width - size.width);
                    self.visualization_model.offset = offset;
                    (Status::Captured, None)
                }
            }
            Event::Keyboard(keyboard::Event::ModifiersChanged(modifiers)) => {
                self.shift_down = modifiers.shift();
                (Status::Ignored, None)
            }
            Event::Keyboard(keyboard::Event::KeyPressed {
                key_code: KeyCode::M,
                modifiers,
            }) => {
                if modifiers.command() {
                    self.visualization_model.chart_mode = match self.visualization_model.chart_mode
                    {
                        ChartMode::RMS => ChartMode::Samples,
                        ChartMode::Samples => ChartMode::RMS,
                    };
                    (Status::Captured, None)
                } else {
                    (Status::Ignored, None)
                }
            }
            Event::Keyboard(keyboard::Event::KeyPressed {
                key_code: KeyCode::Equals,
                modifiers,
            }) => {
                if modifiers.command() {
                    self.visualization_model.zoom_x *= 10.0;
                    self.visualization_model.zoom_x =
                        self.visualization_model.zoom_x.min(100.0).max(1.0);
                    (Status::Captured, None)
                } else {
                    (Status::Ignored, None)
                }
            }
            Event::Keyboard(keyboard::Event::KeyPressed {
                key_code: KeyCode::Minus,
                modifiers,
            }) => {
                if modifiers.command() {
                    self.visualization_model.zoom_x /= 10.0;
                    self.visualization_model.zoom_x =
                        self.visualization_model.zoom_x.min(100.0).max(1.0);
                    (Status::Captured, None)
                } else {
                    (Status::Ignored, None)
                }
            }
            _ => (Status::Ignored, None),
        }
    }

    fn draw(&self, bounds: Rectangle, _cursor: Cursor) -> Vec<Geometry> {
        let zoom_x = self.visualization_model.zoom_x;
        let zoom_y = self.visualization_model.zoom_y;
        let mut frame = Frame::new(bounds.size());
        if let Some(audio_file_model) = &self.audio_file_model {
            let width = frame.width() * zoom_x;
            let height = frame.height() * zoom_y;
            let offset = self.visualization_model.offset.min(width - frame.width());
            frame.translate(Vector::from([-offset, 0.0]));

            match self.visualization_model.chart_mode {
                ChartMode::Samples => {
                    draw_samples_chart(
                        &mut frame,
                        width,
                        height,
                        offset,
                        audio_file_model.samples_len() as f32,
                        audio_file_model.samples().cloned(),
                    );
                }
                ChartMode::RMS => {
                    draw_rms_chart(
                        &mut frame,
                        width,
                        height,
                        offset,
                        audio_file_model.rms_len() as f32,
                        audio_file_model.rms().cloned(),
                    );
                }
            }
        }
        vec![frame.into_geometry()]
    }
}

fn draw_samples_chart(
    frame: &mut Frame,
    width: f32,
    height: f32,
    offset: f32,
    num_samples: f32,
    samples_iterator: impl Iterator<Item = f32>,
) {
    let color = Colors::active_border_color();
    let num_pixels = (width * 8.0).max(1000.0);
    let step_size = ((num_samples / num_pixels) as usize).max(1);
    let mut samples = samples_iterator.collect::<Vec<f32>>();

    let mut path = iced::canvas::path::Builder::new();
    for (index, item) in samples.iter().enumerate().step_by(step_size) {
        let f_index = index as f32;
        let x = (f_index / num_samples) * width;
        let y = (*item as f32) * height / 2.0 + frame.height() / 2.0;

        if x < offset {
            continue;
        }

        if x > frame.width() + offset {
            break;
        }

        if !x.is_finite() {
            continue;
        }

        let point = Point::new(x, y);
        path.line_to(point);
    }
    frame.stroke(&path.build(), Stroke::default().with_color(color));
}

fn draw_rms_chart<'a>(
    frame: &mut Frame,
    width: f32,
    height: f32,
    offset: f32,
    num_samples: f32,
    samples_iterator: impl Iterator<Item = f32>,
) {
    let color = Colors::active_border_color();
    let num_pixels = (width * 2.0).max(1000.0);
    let step_size = ((num_samples / num_pixels) as usize).max(1);
    let mut samples = samples_iterator.collect::<Vec<f32>>();

    let mut path = iced::canvas::path::Builder::new();
    path.line_to(Point::new(0.0, frame.height() / 2.0));
    for (index, item) in samples.iter().enumerate().step_by(step_size) {
        let f_index = index as f32;
        let x = (f_index / num_samples) * width;
        let y = (*item as f32) * height + frame.height() / 2.0;

        if x < offset {
            continue;
        }

        if x > frame.width() + offset {
            break;
        }

        if !x.is_finite() {
            continue;
        }

        path.line_to(Point::new(x, y));
    }
    path.line_to(Point::new(frame.width(), frame.height() / 2.0));
    path.line_to(Point::new(0.0, frame.height() / 2.0));
    frame.fill(&path.build(), Fill::from(color));

    let mut path = iced::canvas::path::Builder::new();
    path.line_to(Point::new(0.0, frame.height() / 2.0));
    for (index, item) in samples.iter().enumerate().step_by(step_size) {
        let f_index = index as f32;
        let x = (f_index / num_samples) * width;
        let y = (-item as f32) * height + frame.height() / 2.0;

        if x < offset {
            continue;
        }

        if x > frame.width() + offset {
            break;
        }

        if !x.is_finite() {
            continue;
        }

        path.line_to(Point::new(x, y));
    }
    path.line_to(Point::new(frame.width(), frame.height() / 2.0));
    path.line_to(Point::new(0.0, frame.height() / 2.0));
    frame.fill(&path.build(), Fill::from(color));
}

pub mod story {
    use audio_processor_testing_helpers::relative_path;
    use iced::Command;

    use audio_processor_file::AudioFileProcessor;
    use audio_processor_iced_storybook::StoryView;
    use audio_processor_traits::AudioProcessorSettings;

    use super::*;

    pub fn default() -> Story {
        Story::default()
    }

    pub struct Story {
        editor: AudioEditorView,
    }

    impl Default for Story {
        fn default() -> Self {
            let mut editor = AudioEditorView::default();
            let settings = AudioProcessorSettings::default();
            log::info!("Reading audio file");
            let audio_file_buffer = get_example_file_buffer(settings);
            log::info!("Building editor model");
            editor.audio_file_model = Some(AudioFileModel::from_buffer(audio_file_buffer));
            log::info!("Starting");
            Self { editor }
        }
    }

    fn get_example_file_buffer(settings: AudioProcessorSettings) -> Vec<f32> {
        let mut processor = AudioFileProcessor::from_path(
            audio_garbage_collector::handle(),
            settings,
            &relative_path!("../../../confirmation.mp3"),
            // &relative_path!("../../../../input-files/synthetizer-loop.mp3"),
        )
        .unwrap();
        processor.prepare(settings);
        let channels = processor.buffer().clone();
        let mut output = vec![];
        for (s1, s2) in channels[0].iter().zip(channels[1].clone()) {
            output.push(s1 + s2);
        }
        output
    }

    impl StoryView<Message> for Story {
        fn update(&mut self, message: Message) -> Command<Message> {
            self.editor.update(message);
            Command::none()
        }

        fn view(&mut self) -> Element<Message> {
            self.editor.view()
        }
    }
}
