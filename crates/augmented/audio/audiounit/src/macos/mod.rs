mod sys;

use cacao::core_foundation::base::OSStatus;
use cacao::foundation::NSNumber;
use cocoa::base::{id, nil, BOOL, YES};
use objc::msg_send;
use std::ffi::c_void;

// /// An audio component. - <https://developer.apple.com/documentation/audiotoolbox/audiocomponent?language=objc>
// #[repr(C)]
// pub struct AudioComponent {
//     private: [u8; 0],
// }
//
// /// A component instance, or object, is an audio unit or audio codec. - <https://developer.apple.com/documentation/audiotoolbox/audiocomponentinstance?language=objc>
// #[repr(C)]
// pub struct AudioComponentInstance {
//     private: [u8; 0],
// }

#[link(name = "AVFAudio", kind = "framework")]
extern "C" {}

pub struct AUAudioUnit {
    reference: id,
}

impl AUAudioUnit {
    /// Instantiate the audio-unit
    pub fn instantiate(description: AudioComponentDescription) -> AUAudioUnit {
        let (tx, rx) = std::sync::mpsc::channel::<(id, id)>();
        let completion_handler = block::ConcreteBlock::new(move |au_audio_unit: id, error: id| {
            unsafe {
                if au_audio_unit != nil {
                    let _: c_void = msg_send![au_audio_unit, retain];
                    let str: id = msg_send![au_audio_unit, componentName];
                    let str = cacao::foundation::NSString::retain(str);
                    let str = str.to_string();
                    println!("Loaded audio-unit: {}", str);
                }
                if error != nil {
                    let _: c_void = msg_send![error, retain];
                }
            }
            tx.send((au_audio_unit, error)).unwrap();
        });
        let completion_handler = completion_handler.copy();

        unsafe {
            let _: c_void = msg_send![
                class!(AUAudioUnit),
                instantiateWithComponentDescription: description
                options:0
                completionHandler:&*completion_handler
            ];
        }
        let (au_audio_unit, error) = rx.recv().unwrap();
        if error != nil {
            let error = avfaudio_sys::NSError(error);
            let str = unsafe { avfaudio_sys::INSError::localizedDescription(&error) };
            let str = cacao::foundation::NSString::retain(str.0);
            let str = str.to_string();
            println!("Error: {}", str);
        }

        unsafe {
            let str: id = msg_send![au_audio_unit, componentName];
            let str = cacao::foundation::NSString::retain(str);
            let str = str.to_string();
            println!("Loaded audio-unit: {}", str);
        }

        AUAudioUnit {
            reference: au_audio_unit,
        }
    }

    /// Create a view controller for the audio-unit
    pub fn request_view_controller(&self) -> id {
        let (tx, rx) = std::sync::mpsc::channel::<id>();
        let completion_handler = block::ConcreteBlock::new(move |obj| {
            tx.send(obj).unwrap();
        });
        let completion_handler = completion_handler.copy();
        unsafe {
            let _: c_void = msg_send![
                self.reference,
                requestViewControllerWithCompletionHandler:&*completion_handler
            ];
        }

        rx.recv().unwrap()
    }

    /// Allocates resources required to render audio.
    ///
    /// Returns an error or nil
    pub fn allocate_render_resources(&self) -> id {
        let error = nil;
        unsafe {
            let _: BOOL = msg_send![self.reference, allocateRenderResourcesAndReturnError: error];
        }
        error
    }

    /// Deallocates resources required to render audio.
    pub fn deallocate_render_resources(&self) {
        unsafe {
            let _: c_void = msg_send![self.reference, deallocateRenderResources];
        }
    }

    /// The block that hosts use to ask the audio unit to render audio.
    pub fn render_block(
        &self,
    ) -> *const block::Block<
        (
            *mut avfaudio_sys::AudioUnitRenderActionFlags,
            *const avfaudio_sys::AudioTimeStamp,
            avfaudio_sys::AUAudioFrameCount,
            avfaudio_sys::NSInteger,
            *mut avfaudio_sys::AudioBufferList,
            avfaudio_sys::AURenderPullInputBlock,
        ),
        avfaudio_sys::AUAudioUnitStatus,
    > {
        unsafe { msg_send![self.reference, renderBlock] }
    }
}

/// Wraps `AVAudioUnitComponent` - <https://developer.apple.com/documentation/avfaudio/avaudiounitcomponent?language=objc>
///
/// A class that provides details about an audio unit such as: type, subtype, manufacturer, and
/// location.
pub struct AVAudioUnitComponent {
    reference: id,
}

impl AVAudioUnitComponent {
    pub fn new(reference: id) -> Self {
        Self { reference }
    }

    // /// The AudioComponent of the audio unit component.
    // pub fn audio_component(&self) -> *mut AudioComponent {
    //     unsafe { msg_send![self.reference, audioComponent] }
    // }

    /// The [`AudioComponentDescription`] of the audio unit component.
    pub fn audio_component_description(&self) -> AudioComponentDescription {
        unsafe { msg_send![self.reference, audioComponentDescription] }
    }

    /// The name of the audio unit component.
    pub fn name(&self) -> String {
        unsafe {
            // NSString*
            let ns_string: id = msg_send![self.reference, name];
            AVAudioUnitComponent::build_string(ns_string)
        }
    }

    /// The name of the manufacturer of the audio unit component.
    pub fn manufacturer_name(&self) -> String {
        unsafe {
            // NSString*
            let ns_string: id = msg_send![self.reference, manufacturerName];
            AVAudioUnitComponent::build_string(ns_string)
        }
    }

    /// The audio unit component type.
    pub fn component_type_name(&self) -> String {
        unsafe {
            // NSString*
            let ns_string: id = msg_send![self.reference, typeName];
            AVAudioUnitComponent::build_string(ns_string)
        }
    }

    /// A string representing the audio unit component version number
    pub fn version_string(&self) -> String {
        unsafe {
            // NSString*
            let ns_string: id = msg_send![self.reference, versionString];
            AVAudioUnitComponent::build_string(ns_string)
        }
    }

    fn build_string(ns_string: id) -> String {
        let ns_string = cacao::foundation::NSString::retain(ns_string);
        ns_string.to_string()
    }

    /// An array of supported architectures.
    pub fn available_architectures(&self) -> Vec<i64> {
        let mut result = vec![];
        unsafe {
            // NSArray*
            let ns_array: id = msg_send![self.reference, availableArchitectures];
            let count = msg_send![ns_array, count];

            for i in 0..count {
                // NSNumber*
                let item: id = msg_send![ns_array, objectAtIndex: i];
                result.push(NSNumber::wrap(item).as_i64())
            }
        }
        result
    }

    /// Whether the audio unit component has a custom view.
    pub fn has_custom_view(&self) -> bool {
        unsafe {
            let result: BOOL = msg_send![self.reference, hasCustomView];
            result == YES
        }
    }

    /// Whether the audio unit component has midi input.
    pub fn has_midi_input(&self) -> bool {
        unsafe {
            let result: BOOL = msg_send![self.reference, hasMIDIInput];
            result == YES
        }
    }

    /// Whether the audio unit component has midi output.
    pub fn has_midi_output(&self) -> bool {
        unsafe {
            let result: BOOL = msg_send![self.reference, hasMIDIOutput];
            result == YES
        }
    }
}

/// Wraps `AudioComponentDescription` - <https://developer.apple.com/documentation/audiotoolbox/audiocomponentdescription?language=objc>
///
/// Identifying information for an audio component.
#[derive(Debug, Clone, Copy, PartialEq)]
#[repr(C)]
#[allow(non_snake_case)]
pub struct AudioComponentDescription {
    /// A unique 4-byte code identifying the interface for the component.
    pub componentType: u32,
    /// A 4-byte code that you can use to indicate the purpose of a component. For example, you
    /// could use lpas or lowp as a mnemonic indication that an audio unit is a low-pass filter.
    pub componentSubType: u32,
    /// The unique vendor identifier, registered with Apple, for the audio component.
    pub componentManufacturer: u32,
    /// Set this value to zero.
    pub componentFlags: u32,
    /// Set this value to zero.
    pub componentFlagsMask: u32,
}

/// Wraps `AVAudioUnitComponentManager` - <https://developer.apple.com/documentation/avfaudio/avaudiounitcomponentmanager?language=objc>
///
/// An object that provides a way to search and query audio components that are registered with the
/// system.
///
/// ## Overview
/// > The component manager has methods to find various information about the audio components without
/// > opening them. Currently, only audio components that are audio units can be searched.
/// >
/// > The class also supports predefined system tags and arbitrary user tags. Each audio unit can be
/// > tagged as part of its definition. AudioUnit Hosts such as Logic or GarageBand can present
/// > groupings of audio units based on the tags.
///
/// See the link above for more information.
pub struct AVAudioUnitComponentManager {
    reference: id,
}

impl AVAudioUnitComponentManager {
    /// Returns the shared component manager.
    pub fn shared() -> AVAudioUnitComponentManager {
        AVAudioUnitComponentManager {
            reference: unsafe {
                msg_send![
                    class!(AVAudioUnitComponentManager),
                    sharedAudioUnitComponentManager
                ]
            },
        }
    }

    /// Return all [`AudioUnit`] component information
    pub fn all_components(&self) -> Vec<AVAudioUnitComponent> {
        let mut result = vec![];
        unsafe {
            // NSPredicate
            let predicate: id = msg_send![class!(NSPredicate), predicateWithValue: YES];

            // NSArray
            let components_array: id =
                msg_send![self.reference, componentsMatchingPredicate: predicate];
            let count = msg_send![components_array, count];

            for i in 0..count {
                let item: id = msg_send![components_array, objectAtIndex: i];
                let unit = AVAudioUnitComponent::new(item);
                result.push(unit);
            }
        }

        result
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use cocoa::quartzcore::transaction::set_value_for_key;

    #[test]
    fn test_list_all_components() {
        let mut manager = AVAudioUnitComponentManager::shared();
        let components = manager.all_components();
        for component in components {
            println!("Name: {}", component.name());
        }
    }

    #[test]
    fn test_list_all_components_and_archs() {
        let mut manager = AVAudioUnitComponentManager::shared();
        let components = manager.all_components();
        let archs = components[0].available_architectures();
        println!("{:?}", archs);
    }

    #[test]
    fn test_name_and_desc_strings() {
        let mut manager = AVAudioUnitComponentManager::shared();
        let components = manager.all_components();
        let component = &components[0];
        println!("Name: {:?}", component.name());
        println!("Manufacturer: {:?}", component.manufacturer_name());
        println!("Version: {:?}", component.version_string());
        println!("Type: {:?}", component.component_type_name());
    }

    #[test]
    fn test_query_description() {
        let mut manager = AVAudioUnitComponentManager::shared();
        let components = manager.all_components();
        let component = &components[0];
        println!("Description: {:?}", component.audio_component_description());
    }

    #[test]
    fn test_query_capabilities() {
        let mut manager = AVAudioUnitComponentManager::shared();
        let components = manager.all_components();
        let component = &components[0];
        println!("{}", component.has_custom_view());
        println!("{}", component.has_midi_input());
        println!("{}", component.has_midi_output());
    }

    #[test]
    fn test_create_unit() {
        let mut manager = AVAudioUnitComponentManager::shared();
        let components = manager.all_components();
        let component = &components[5];
        let unit = AUAudioUnit::instantiate(component.audio_component_description());
        let render_block = unit.render_block();
        let mut render_action_flags =
            avfaudio_sys::AudioUnitRenderActionFlags_kAudioOfflineUnitRenderAction_Render;
        let audio_timestamp = avfaudio_sys::AudioTimeStamp {
            mSampleTime: 0.0,
            mHostTime: 0,
            mRateScalar: 0.0,
            mWordClockTime: 0,
            mSMPTETime: avfaudio_sys::SMPTETime {
                mSubframes: 0,
                mSubframeDivisor: 0,
                mCounter: 0,
                mType: 0,
                mFlags: 0,
                mHours: 0,
                mMinutes: 0,
                mSeconds: 0,
                mFrames: 0,
            },
            mFlags: 0,
            mReserved: 0,
        };
        let audio_frame_count = 0;
        let output_bus_number = 1;
        let vec_buffer: Vec<f32> = Vec::new();
        let buffer = avfaudio_sys::AudioBuffer {
            mNumberChannels: 1,
            mDataByteSize: 8,
            mData: vec_buffer.as_ptr() as *mut c_void,
        };
        let mut output_data = avfaudio_sys::AudioBufferList {
            mNumberBuffers: 1,
            mBuffers: [buffer],
        };
        let pull_input_block = block::ConcreteBlock::new(
            |flags: *mut u32,
             timestamp: *const avfaudio_sys::AudioTimeStamp,
             frame_count: u32,
             bus_number: i64,
             buffer_list: *mut avfaudio_sys::AudioBufferList| {
                // TODO -> This is how input is provided.
                0
            },
        );
        let pull_input_block = pull_input_block.copy();

        // let pull_input_block_ptr: avfaudio_sys::AURenderPullInputBlock =
        //     &*pull_input_block as *const _;

        unsafe {
            render_block.as_ref().unwrap().call((
                // *mut avfaudio_sys::AudioUnitRenderActionFlags,
                &mut render_action_flags as *mut u32,
                // *const avfaudio_sys::AudioTimeStamp,
                &audio_timestamp as *const avfaudio_sys::AudioTimeStamp,
                // avfaudio_sys::AUAudioFrameCount,
                audio_frame_count,
                // avfaudio_sys::NSInteger,
                output_bus_number,
                // *mut avfaudio_sys::AudioBufferList,
                &mut output_data as *mut avfaudio_sys::AudioBufferList,
                // avfaudio_sys::AURenderPullInputBlock,
                &*pull_input_block,
            ));
        }
    }
}
