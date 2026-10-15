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
        pub fn crush(&self) {
            unsafe { bb_audio_crush() }
        }
    }
}
#[cfg(target_arch = "wasm32")]
pub use web::Audio;
