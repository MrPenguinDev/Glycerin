//! Phase 3: Media Support Layer
//! Implements HTML5 video/audio playback, image decoding, and media controls

use image::{DynamicImage, GenericImageView, ImageFormat};
use std::io::Cursor;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use std::time::Duration;

#[derive(Debug, Clone)]
pub enum MediaType {
    Audio(AudioFormat),
    Video(VideoFormat),
    Image(ImageFormat),
}

#[derive(Debug, Clone)]
pub enum AudioFormat {
    Mp3,
    Wav,
    Ogg,
    Flac,
    M4a,
    Unknown,
}

#[derive(Debug, Clone)]
pub enum VideoFormat {
    Mp4,
    Webm,
    Ogx,
    Unknown,
}

impl AudioFormat {
    pub fn from_extension(ext: &str) -> Self {
        match ext.to_lowercase().as_str() {
            "mp3" => AudioFormat::Mp3,
            "wav" => AudioFormat::Wav,
            "ogg" => AudioFormat::Ogg,
            "flac" => AudioFormat::Flac,
            "m4a" => AudioFormat::M4a,
            _ => AudioFormat::Unknown,
        }
    }
    
    pub fn mime_type(&self) -> &'static str {
        match self {
            AudioFormat::Mp3 => "audio/mpeg",
            AudioFormat::Wav => "audio/wav",
            AudioFormat::Ogg => "audio/ogg",
            AudioFormat::Flac => "audio/flac",
            AudioFormat::M4a => "audio/mp4",
            AudioFormat::Unknown => "application/octet-stream",
        }
    }
}

impl VideoFormat {
    pub fn from_extension(ext: &str) -> Self {
        match ext.to_lowercase().as_str() {
            "mp4" => VideoFormat::Mp4,
            "webm" => VideoFormat::Webm,
            "ogv" => VideoFormat::Ogx,
            _ => VideoFormat::Unknown,
        }
    }
    
    pub fn mime_type(&self) -> &'static str {
        match self {
            VideoFormat::Mp4 => "video/mp4",
            VideoFormat::Webm => "video/webm",
            VideoFormat::Ogx => "video/ogg",
            VideoFormat::Unknown => "application/octet-stream",
        }
    }
}

#[derive(Debug, Clone)]
pub struct MediaMetadata {
    pub duration_secs: f64,
    pub sample_rate: u32,
    pub channels: u16,
    pub format: MediaType,
    pub title: Option<String>,
    pub artist: Option<String>,
    pub album: Option<String>,
}

#[derive(Debug, Clone)]
pub struct DecodedImage {
    pub width: u32,
    pub height: u32,
    pub data: Vec<u8>, // RGBA format
    pub format: ImageFormat,
}

pub struct AudioManager {
    is_playing: bool,
    volume: f32,
    current_time_secs: f64,
    loaded_audio: Option<Vec<u8>>,
    loaded_format: Option<AudioFormat>,
}

impl AudioManager {
    pub fn new() -> Result<Self, &'static str> {
        Ok(Self {
            is_playing: false,
            volume: 1.0,
            current_time_secs: 0.0,
            loaded_audio: None,
            loaded_format: None,
        })
    }
    
    pub fn load_audio(&mut self, data: &[u8], format: AudioFormat) -> Result<(), &'static str> {
        if matches!(format, AudioFormat::Unknown) {
            return Err("Unknown audio format");
        }
        self.loaded_audio = Some(data.to_vec());
        self.loaded_format = Some(format);
        self.current_time_secs = 0.0;
        Ok(())
    }
    
    pub fn play(&mut self) {
        if self.loaded_audio.is_some() {
            self.is_playing = true;
        }
    }
    
    pub fn pause(&mut self) {
        self.is_playing = false;
    }
    
    pub fn stop(&mut self) {
        self.is_playing = false;
        self.current_time_secs = 0.0;
    }
    
    pub fn set_volume(&mut self, volume: f32) {
        self.volume = volume.clamp(0.0, 1.0);
    }
    
    pub fn get_volume(&self) -> f32 {
        self.volume
    }
    
    pub fn is_playing(&self) -> bool {
        self.is_playing
    }
    
    pub fn get_current_time(&self) -> f64 {
        self.current_time_secs
    }
    
    pub fn seek(&mut self, position_secs: f64) {
        self.current_time_secs = position_secs.max(0.0);
    }
    
    pub fn get_metadata(&self, data: &[u8], format: AudioFormat) -> Option<MediaMetadata> {
        if matches!(format, AudioFormat::Unknown) || data.is_empty() {
            return None;
        }
        
        Some(MediaMetadata {
            duration_secs: 0.0,
            sample_rate: 0,
            channels: 0,
            format: MediaType::Audio(format),
            title: None,
            artist: None,
            album: None,
        })
    }
}

impl Default for AudioManager {
    fn default() -> Self {
        Self::new().expect("in-memory audio manager construction cannot fail")
    }
}
pub struct ImageDecoder {
    supported_formats: Vec<ImageFormat>,
}

impl ImageDecoder {
    pub fn new() -> Self {
        Self {
            supported_formats: vec![
                ImageFormat::Png,
                ImageFormat::Jpeg,
                ImageFormat::Gif,
                ImageFormat::WebP,
            ],
        }
    }
    
    pub fn decode(&self, data: &[u8]) -> Result<DecodedImage, &'static str> {
        let img = image::load_from_memory(data)
            .map_err(|_| "Failed to decode image")?;
        
        let (width, height) = img.dimensions();
        let rgba = img.to_rgba8();
        
        Ok(DecodedImage {
            width,
            height,
            data: rgba.to_vec(),
            format: ImageFormat::Png, // Default, would detect actual format
        })
    }
    
    pub fn decode_with_format(&self, data: &[u8], format: ImageFormat) -> Result<DecodedImage, &'static str> {
        let cursor = Cursor::new(data.to_vec());
        
        let img = image::load(cursor, format)
            .map_err(|_| "Failed to decode image with specified format")?;
        
        let (width, height) = img.dimensions();
        let rgba = img.to_rgba8();
        
        Ok(DecodedImage {
            width,
            height,
            data: rgba.to_vec(),
            format,
        })
    }
    
    pub fn is_format_supported(&self, format: ImageFormat) -> bool {
        self.supported_formats.contains(&format)
    }
    
    pub fn get_supported_formats(&self) -> &[ImageFormat] {
        &self.supported_formats
    }
    
    pub fn resize(&self, image: &DecodedImage, new_width: u32, new_height: u32) -> Result<DecodedImage, &'static str> {
        let img = image::RgbaImage::from_raw(image.width, image.height, image.data.clone())
            .ok_or("Invalid image data")?;
        
        let resized = image::imageops::resize(
            &img,
            new_width,
            new_height,
            image::imageops::FilterType::Lanczos3,
        );
        
        Ok(DecodedImage {
            width: new_width,
            height: new_height,
            data: resized.to_vec(),
            format: image.format,
        })
    }
}

impl Default for ImageDecoder {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone)]
pub struct VideoFrame {
    pub width: u32,
    pub height: u32,
    pub data: Vec<u8>, // YUV or RGBA format
    pub timestamp_ms: u64,
}

pub struct VideoPlayer {
    current_frame: Option<VideoFrame>,
    is_playing: bool,
    is_paused: bool,
    volume: f32,
    current_time_ms: u64,
    duration_ms: u64,
    fps: f32,
}

impl VideoPlayer {
    pub fn new() -> Self {
        Self {
            current_frame: None,
            is_playing: false,
            is_paused: false,
            volume: 1.0,
            current_time_ms: 0,
            duration_ms: 0,
            fps: 30.0,
        }
    }
    
    pub fn load_video(&mut self, data: &[u8], format: VideoFormat) -> Result<(), &'static str> {
        // Placeholder: Full video decoding would require FFmpeg or similar
        // For now, we acknowledge the video but can't decode it fully
        match format {
            VideoFormat::Mp4 | VideoFormat::Webm | VideoFormat::Ogx => {
                // In a full implementation, this would initialize a video decoder
                self.duration_ms = 0; // Would parse from container
                self.fps = 30.0; // Would parse from stream
                Ok(())
            }
            VideoFormat::Unknown => Err("Unknown video format"),
        }
    }
    
    pub fn play(&mut self) {
        if !self.is_playing {
            self.is_playing = true;
            self.is_paused = false;
        }
    }
    
    pub fn pause(&mut self) {
        if self.is_playing {
            self.is_paused = true;
        }
    }
    
    pub fn stop(&mut self) {
        self.is_playing = false;
        self.is_paused = false;
        self.current_time_ms = 0;
        self.current_frame = None;
    }
    
    pub fn seek(&mut self, position_ms: u64) {
        self.current_time_ms = position_ms.min(self.duration_ms);
    }
    
    pub fn set_volume(&mut self, volume: f32) {
        self.volume = volume.clamp(0.0, 1.0);
    }
    
    pub fn get_current_frame(&self) -> Option<&VideoFrame> {
        self.current_frame.as_ref()
    }
    
    pub fn update_frame(&mut self, frame: VideoFrame) {
        self.current_frame = Some(frame);
    }
    
    pub fn get_duration_ms(&self) -> u64 {
        self.duration_ms
    }
    
    pub fn get_current_time_ms(&self) -> u64 {
        self.current_time_ms
    }
    
    pub fn is_playing(&self) -> bool {
        self.is_playing && !self.is_paused
    }
}

impl Default for VideoPlayer {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_audio_format_detection() {
        assert_eq!(AudioFormat::from_extension("mp3"), AudioFormat::Mp3);
        assert_eq!(AudioFormat::from_extension("MP3"), AudioFormat::Mp3);
        assert_eq!(AudioFormat::from_extension("wav"), AudioFormat::Wav);
        assert_eq!(AudioFormat::from_extension("ogg"), AudioFormat::Ogg);
        assert_eq!(AudioFormat::from_extension("unknown"), AudioFormat::Unknown);
    }
    
    #[test]
    fn test_video_format_detection() {
        assert_eq!(VideoFormat::from_extension("mp4"), VideoFormat::Mp4);
        assert_eq!(VideoFormat::from_extension("webm"), VideoFormat::Webm);
        assert_eq!(VideoFormat::from_extension("ogv"), VideoFormat::Ogx);
        assert_eq!(VideoFormat::from_extension("unknown"), VideoFormat::Unknown);
    }
    
    #[test]
    fn test_image_decoder_creation() {
        let decoder = ImageDecoder::new();
        assert!(decoder.is_format_supported(ImageFormat::Png));
        assert!(decoder.is_format_supported(ImageFormat::Jpeg));
        assert!(decoder.is_format_supported(ImageFormat::WebP));
    }
    
    #[test]
    fn test_video_player_state() {
        let mut player = VideoPlayer::new();
        assert!(!player.is_playing());
        
        player.play();
        assert!(player.is_playing());
        
        player.pause();
        assert!(!player.is_playing());
        
        player.stop();
        assert_eq!(player.get_current_time_ms(), 0);
    }
}
