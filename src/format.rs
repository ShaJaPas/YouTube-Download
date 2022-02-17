#![allow(dead_code)]

use std::fmt::{Display, Formatter};

#[derive(Debug, Eq, PartialEq)]
pub struct Format<'a> {
    itag: i32,
    ext: &'a str,
    height: i32,
    fps: u32,
    v_codec: VCodec,
    a_codec: ACodec,
    audio_bitrate: i32,
    is_dash_container: bool,
    is_hls_content: bool,
}

impl Format<'_> {
    pub fn new(
        itag: i32,
        ext: &str,
        height: i32,
        fps: u32,
        v_codec: VCodec,
        a_codec: ACodec,
        audio_bitrate: i32,
        is_dash_container: bool,
        is_hls_content: bool,
    ) -> Format {
        Format {
            itag,
            ext,
            height,
            fps,
            v_codec,
            a_codec,
            audio_bitrate,
            is_dash_container,
            is_hls_content,
        }
    }

    /** Get the frames per second */
    pub fn get_fps(&self) -> u32 {
        self.fps
    }

    /** Audio bitrate in kbit/s or -1 if there is no audio track.*/
    pub fn get_audio_bitrate(&self) -> i32 {
        self.audio_bitrate
    }

    /** An identifier used by youtube for different formats.*/
    pub fn get_itag(&self) -> i32 {
        self.itag
    }

    /**The file extension and conainer format like "mp4"*/
    pub fn get_extension(&self) -> &str {
        self.ext
    }

    /** The pixel height of the video stream or -1 for audio files.*/
    pub fn get_height(&self) -> i32 {
        self.height
    }

    /** The audio codec of format*/
    pub fn get_audio_codec(&self) -> &ACodec {
        &self.a_codec
    }

    /** The video codec of format*/
    pub fn get_video_codec(&self) -> &VCodec {
        &self.v_codec
    }
}

impl ToString for Format<'_> {
    fn to_string(&self) -> String {
        return format!(
            "Format (\
            itag={}, \
            ext=\'{}\', \
            height={},\
            fps={}, \
            vCodec={}, \
            aCodec={}, \
            audioBitrate={}, \
            isDashContainer={}, \
            isHlsContent={})",
            self.itag,
            self.ext,
            self.height,
            self.fps,
            self.v_codec,
            self.a_codec,
            self.audio_bitrate,
            self.is_dash_container,
            self.is_hls_content
        );
    }
}

#[derive(Debug, Eq, PartialEq)]
pub enum VCodec {
    H263,
    H264,
    MPEG4,
    VP8,
    VP9,
    NONE,
}

#[derive(Debug, Eq, PartialEq)]
pub enum ACodec {
    MP3,
    AAC,
    VORBIS,
    OPUS,
    NONE,
}

impl Display for ACodec {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self)
    }
}

impl Display for VCodec {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self)
    }
}
