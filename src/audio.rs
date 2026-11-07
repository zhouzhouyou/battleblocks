#[cfg(not(target_arch = "wasm32"))]
#[path = "audio/speech.rs"]
mod speech;
#[cfg(not(target_arch = "wasm32"))]
#[path = "audio/voice_filter.rs"]
mod voice_filter;

#[cfg(not(target_arch = "wasm32"))]
#[path = "audio/native.rs"]
mod native;
#[cfg(not(target_arch = "wasm32"))]
pub use native::Audio;
#[cfg(target_arch = "wasm32")]
mod web {
    unsafe extern "C" {
        fn bb_audio_init(music: u32, effects: u32);
        fn bb_audio_playing(value: u32);
        fn bb_audio_muted(value: u32);
        fn bb_audio_crush();
        fn bb_audio_volume(value: u32);
        fn bb_audio_portal();
        fn bb_audio_speak(ptr: *const u8, len: usize, zh: u32);
        fn bb_audio_stop_speech();
    }
    pub struct Audio;
    impl Audio {
        pub fn new(music: bool, effects: bool) -> Result<Self, String> {
            unsafe {
                bb_audio_init(music as u32, effects as u32);
            }
            Ok(Self)
        }
        pub fn playing(&self, v: bool) {
            unsafe { bb_audio_playing(v as u32) }
        }
        pub fn muted(&self, v: bool) {
            unsafe { bb_audio_muted(v as u32) }
        }
        pub fn volume(&self, value: u32) {
            unsafe { bb_audio_volume(value.min(100)) }
        }
        pub fn portal(&self) {
            unsafe { bb_audio_portal() }
        }
        pub fn speak(&self, text: &str, zh: bool) {
            unsafe { bb_audio_speak(text.as_ptr(), text.len(), zh as u32) }
        }
        pub fn stop_speech(&self) {
            unsafe { bb_audio_stop_speech() }
        }
        pub fn crush(&self) {
            unsafe { bb_audio_crush() }
        }
    }
}
#[cfg(target_arch = "wasm32")]
pub use web::Audio;

// Procedural transit sound: a short rising, modulated chime. No bundled sample.
#[cfg(not(target_arch = "wasm32"))]
fn portal_samples(rate: u32) -> Vec<f32> {
    (0..(rate as f32 * 0.28) as usize)
        .map(|n| {
            let t = n as f32 / rate as f32;
            let envelope = (t / 0.015).min(1.) * (1. - t / 0.28).max(0.).powi(2);
            let phase = std::f32::consts::TAU * (260. * t + 1800. * t * t);
            envelope * (phase.sin() + 0.3 * (phase * 1.5).sin()) * 0.2
        })
        .collect()
}
