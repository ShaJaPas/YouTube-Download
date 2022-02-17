use anyhow::anyhow;
use std::collections::HashMap;

use crate::format::ACodec::*;
use crate::format::VCodec::*;
use crate::format::*;
use regex::Regex;

lazy_static! {
    static ref FORMAT_MAP: HashMap<i32, Format<'static>> = {
        let mut m = HashMap::new();
        // Video and Audio
        m.insert(17, Format::new(17, "3gp", 144, 30, MPEG4, AAC, 24, false, false));
        m.insert(36, Format::new(36, "3gp", 240, 30, MPEG4, AAC, 32, false, false));
        m.insert(5, Format::new(5, "flv", 240, 30, H263, MP3, 64, false, false));
        m.insert(43, Format::new(43, "webm", 360, 30, VP8, VORBIS, 128, false, false));
        m.insert(18, Format::new(18, "mp4", 360, 30, H264, AAC, 96, false, false));
        m.insert(22, Format::new(22, "mp4", 720, 30, H264, AAC, 192, false, false));

        // Dash Video
        m.insert(160, Format::new(160, "mp4", 144, 30, H264, ACodec::NONE, -1, true, false));
        m.insert(133, Format::new(133, "mp4", 240, 30, H264, ACodec::NONE, -1, true, false));
        m.insert(134, Format::new(134, "mp4", 360, 30, H264, ACodec::NONE, -1, true, false));
        m.insert(135, Format::new(135, "mp4", 480, 30, H264, ACodec::NONE, -1, true, false));
        m.insert(136, Format::new(136, "mp4", 720, 30, H264, ACodec::NONE, -1, true, false));
        m.insert(137, Format::new(137, "mp4", 1080, 30, H264, ACodec::NONE, -1, true, false));
        m.insert(264, Format::new(264, "mp4", 1440, 30, H264, ACodec::NONE, -1, true, false));
        m.insert(266, Format::new(266, "mp4", 2160, 30, H264, ACodec::NONE, -1, true, false));

        m.insert(298, Format::new(298, "mp4", 720, 60, H264, ACodec::NONE, -1, true, false));
        m.insert(299, Format::new(299, "mp4", 1080, 30, H264, ACodec::NONE, -1, true, false));

        // Dash Audio
        m.insert(140, Format::new(140, "m4a", -1, 30, VCodec::NONE, AAC, 128, true, false));
        m.insert(141, Format::new(141, "m4a", -1, 30, VCodec::NONE, AAC, 256, true, false));
        m.insert(256, Format::new(256, "m4a", -1, 30, VCodec::NONE, AAC, 192, true, false));
        m.insert(258, Format::new(258, "m4a", -1, 30, VCodec::NONE, AAC, 384, true, false));

        // WEBM Dash Video
        m.insert(278, Format::new(278, "webm", 144, 30, VP9, ACodec::NONE, -1, true, false));
        m.insert(242, Format::new(242, "webm", 240, 30, VP9, ACodec::NONE, -1, true, false));
        m.insert(243, Format::new(243, "webm", 360, 30, VP9, ACodec::NONE, -1, true, false));
        m.insert(244, Format::new(244, "webm", 480, 30, VP9, ACodec::NONE, -1, true, false));
        m.insert(247, Format::new(247, "webm", 720, 30, VP9, ACodec::NONE, -1, true, false));
        m.insert(248, Format::new(248, "webm", 1080, 30, VP9, ACodec::NONE, -1, true, false));
        m.insert(271, Format::new(271, "webm", 1440, 30, VP9, ACodec::NONE, -1, true, false));
        m.insert(313, Format::new(313, "webm", 2160, 30, VP9, ACodec::NONE, -1, true, false));

        m.insert(302, Format::new(302, "webm", 720, 60, VP9, ACodec::NONE, -1, true, false));
        m.insert(308, Format::new(308, "webm", 1440, 60, VP9, ACodec::NONE, -1, true, false));
        m.insert(303, Format::new(303, "webm", 1080, 60, VP9, ACodec::NONE, -1, true, false));
        m.insert(315, Format::new(315, "webm", 2160, 60, VP9, ACodec::NONE, -1, true, false));

        // WEBM Dash Audio
        m.insert(171, Format::new(171, "webm", -1, 30, VCodec::NONE, VORBIS, 128, true, false));

        m.insert(249, Format::new(249, "webm", -1, 30, VCodec::NONE, OPUS, 48, true, false));
        m.insert(250, Format::new(250, "webm", -1, 30, VCodec::NONE, OPUS, 64, true, false));
        m.insert(251, Format::new(251, "webm", -1, 30, VCodec::NONE, OPUS, 160, true, false));

        // HLS Live Stream
        m.insert(91, Format::new(91, "mp4", 144, 30, H264, AAC, 48, false, true));
        m.insert(92, Format::new(92, "mp4", 240, 30, H264, AAC, 128, false, true));
        m.insert(93, Format::new(93, "mp4", 360, 30, H264, AAC, 128, false, true));
        m.insert(94, Format::new(94, "mp4", 480, 30, H264, AAC, 256, false, true));
        m.insert(95, Format::new(95, "mp4", 720, 30, H264, AAC, 256, false, true));
        m.insert(96, Format::new(96, "mp4", 1080, 30, H264, AAC, 256, false, true));

        m
    };
}

pub async fn get_stream_urls(uri: &str) -> anyhow::Result<HashMap<i32, (String, &Format<'_>)>> {
    let mut result = HashMap::new();

    let player_response = Regex::new("var ytInitialPlayerResponse\\s*=\\s*(\\{.+?\\})\\s*;")?;
    let sig_enc_url = Regex::new("url=(.+?)(\\u0026|$)")?;
    let signature = Regex::new("s=(.+?)(\\u0026|$)")?;

    let client = reqwest::Client::new();
    let text = client.get(uri).send().await?.text().await?;

    let text = player_response
        .captures(&text)
        .ok_or(anyhow!("No captures were found on html page"))?
        .get(1)
        .ok_or(anyhow!("No captures were found on html page"))?
        .as_str();

    let value = json::parse(text)?;

    let formats = &value["streamingData"]["formats"];
    for i in 0..formats.len() {
        let format = &formats[i];

        let _type = &format["type"];

        if !_type.is_null()
            && _type
                .as_str()
                .ok_or(anyhow!("Cannot convert JsonValue to a string"))?
                .eq("FORMAT_STREAM_TYPE_OTF")
        {
            continue;
        }

        let itag = format["itag"]
            .as_i32()
            .ok_or(anyhow!("Cannot convert JsonValue to an int"))?;

        if FORMAT_MAP.contains_key(&itag) {
            if !format["url"].is_null() {
                let url = format["url"]
                    .as_str()
                    .ok_or(anyhow!("Cannot convert JsonValue to a string"))?
                    .replace("\\u0026", "&");
                result.insert(itag, (url, FORMAT_MAP.get(&itag).unwrap()));
            } else if !format["signatureCipher"].is_null() {
                let cipher = format["signatureCipher"]
                    .as_str()
                    .ok_or(anyhow!("Cannot convert JsonValue to a string"))?;

                if sig_enc_url.is_match(cipher) && signature.is_match(cipher) {
                    let url = sig_enc_url
                        .captures(&cipher)
                        .ok_or(anyhow!("No captures were found on cipher"))?
                        .get(1)
                        .ok_or(anyhow!("No captures were found on html cipher"))?
                        .as_str();
                    let url = urlencoding::decode(url)?.to_string();

                    let signature = signature
                        .captures(&cipher)
                        .ok_or(anyhow!("No captures were found on cipher"))?
                        .get(1)
                        .ok_or(anyhow!("No captures were found on html cipher"))?
                        .as_str();
                    let _signature = urlencoding::decode(signature)?.to_string(); //todo

                    result.insert(itag, (url, FORMAT_MAP.get(&itag).unwrap()));
                }
            }
        }
    }

    let adaptive_formats = &value["streamingData"]["adaptiveFormats"];

    for i in 0..adaptive_formats.len() {
        let format = &adaptive_formats[i];
        let _type = &format["type"];

        if !_type.is_null()
            && _type
                .as_str()
                .ok_or(anyhow!("Cannot convert JsonValue to a string"))?
                .eq("FORMAT_STREAM_TYPE_OTF")
        {
            continue;
        }

        let itag = format["itag"]
            .as_i32()
            .ok_or(anyhow!("Cannot convert JsonValue to an int"))?;

        if FORMAT_MAP.contains_key(&itag) {
            if !format["url"].is_null() {
                let url = format["url"]
                    .as_str()
                    .ok_or(anyhow!("Cannot convert JsonValue to a string"))?
                    .replace("\\u0026", "&");
                result.insert(itag, (url, FORMAT_MAP.get(&itag).unwrap()));
            } else if !format["signatureCipher"].is_null() {
                let cipher = format["signatureCipher"]
                    .as_str()
                    .ok_or(anyhow!("Cannot convert JsonValue to a string"))?;

                if sig_enc_url.is_match(cipher) && signature.is_match(cipher) {
                    let url = sig_enc_url
                        .captures(&cipher)
                        .ok_or(anyhow!("No captures were found on cipher"))?
                        .get(1)
                        .ok_or(anyhow!("No captures were found on html cipher"))?
                        .as_str();
                    let url = urlencoding::decode(url)?.to_string();

                    let signature = signature
                        .captures(&cipher)
                        .ok_or(anyhow!("No captures were found on cipher"))?
                        .get(1)
                        .ok_or(anyhow!("No captures were found on html cipher"))?
                        .as_str();
                    let _signature = urlencoding::decode(signature)?.to_string(); //todo

                    result.insert(itag, (url, FORMAT_MAP.get(&itag).unwrap()));
                }
            }
        }
    }
    Ok(result)
}
