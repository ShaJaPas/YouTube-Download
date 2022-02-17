#[macro_use]
extern crate lazy_static;

mod format;
mod help;

use crate::format::Format;
use anyhow::anyhow;
use clap::{Parser, Subcommand};
use futures_util::StreamExt;
use indicatif::{ProgressBar, ProgressStyle};
use regex::Regex;
use std::fs::File;
use std::io::{stdin, stdout, Write};
use std::process::exit;

/// YouTube video downloader, written in Rust
#[derive(Parser, Debug)]
#[clap(name = "yt_download")]
#[clap(about = "A YouTube video downloader, for people who wants to download media from YouTube")]
struct Args {
    #[clap(subcommand)]
    command: Commands,
}

#[derive(Subcommand, Debug)]
enum Commands {
    ///Downloads a media from YouTube with given link
    Download {
        ///Link to YouTube video
        #[clap(required = true)]
        url: String,
    },
}

async fn download_file(url: &str, name: &str) -> anyhow::Result<()> {
    let client = reqwest::Client::new();
    let response = client.get(url).send().await?;

    let total_size = response
        .content_length()
        .ok_or(anyhow!("No content length"))?;
    let pb = ProgressBar::new(total_size);
    pb.set_style(ProgressStyle::default_bar()
        .template("{msg}\n{spinner:.green} [{elapsed_precise}] [{wide_bar:.cyan/blue}] {bytes}/{total_bytes} ({bytes_per_sec}, {eta})")
        .progress_chars("█ "));

    let mut stream = response.bytes_stream();

    let mut file = File::create(name)?;
    let mut downloaded: u64 = 0;

    while let Some(chunk) = stream.next().await {
        let chunk = chunk?;
        file.write_all(&chunk)?;
        let new = std::cmp::min(downloaded + (chunk.len() as u64), total_size);
        downloaded = new;
        pb.set_position(new);
    }
    std::io::stdout().flush()?;
    pb.finish_with_message(format!("Downloaded to \"{}\"", name));

    Ok(())
}

fn read_number() -> i32 {
    let mut input = String::new();
    stdin().read_line(&mut input).unwrap();
    let res = input.trim().parse::<i32>();
    if res.is_err() {
        println!("Not a number!");
        exit(0);
    }
    return res.unwrap();
}

#[tokio::main(flavor = "current_thread")]
async fn main() {
    let youtube_page_link =
        Regex::new("(http|https)://(www\\.|m.|)youtube\\.com/watch\\?v=(.+?)( |\\z|&)").unwrap();
    let youtube_page_short_link =
        Regex::new("(http|https)://(www\\.|)youtu.be/(.+?)( |\\z|&)").unwrap();
    let args = Args::parse();
    match args.command {
        Commands::Download { url } => {
            if youtube_page_link.is_match(&url) || youtube_page_short_link.is_match(&url) {
                let links = help::get_stream_urls(&url).await;
                if links.is_err() {
                    println!(
                        "An error occurs during html parse!\n{:?}",
                        links.err().unwrap()
                    );
                } else {
                    let links = links.unwrap();
                    if links.is_empty() {
                        println!(
                            "No media links were found! Maybe this video has some limitations"
                        );
                    } else {
                        let mut video: Vec<&(String, &Format)> = links
                            .iter()
                            .filter(|a| a.1 .1.get_height() != -1)
                            .map(|a| a.1)
                            .collect();
                        let mut audio: Vec<&(String, &Format)> = links
                            .iter()
                            .filter(|a| a.1 .1.get_height() == -1)
                            .map(|a| a.1)
                            .collect();
                        let client = reqwest::Client::new();
                        let mut delete = Vec::new();
                        for i in 0..video.len() {
                            if client.head(&video[i].0).send().await.unwrap().status() != 200 {
                                delete.push(i);
                            }
                        }
                        let mut m = 0;
                        for x in delete {
                            video.remove(x - m);
                            m += 1;
                        }
                        let mut delete = Vec::new();
                        for i in 0..audio.len() {
                            if client.head(&audio[i].0).send().await.unwrap().status() != 200 {
                                delete.push(i);
                            }
                        }
                        let mut m = 0;
                        for x in delete {
                            audio.remove(x - m);
                            m += 1;
                        }
                        video.sort_by(|a, b| {
                            a.1.get_height().partial_cmp(&b.1.get_height()).unwrap()
                        });
                        audio.sort_by(|a, b| {
                            a.1.get_audio_bitrate()
                                .partial_cmp(&b.1.get_audio_bitrate())
                                .unwrap()
                        });

                        println!("Choose what do you want to download (1-2):");
                        println!("1) Video ({} formats)", video.len());
                        println!("2) Audio ({} formats)", audio.len());
                        let number = read_number();
                        if (1..=2).contains(&number) {
                            match number {
                                1 => {
                                    if video.is_empty() {
                                        exit(0);
                                    }
                                    println!(
                                        "Choose which file do you want to download (1-{}):",
                                        video.len()
                                    );
                                    for i in 0..video.len() {
                                        println!("{}) Format = {}, Resolution = {}p, Fps = {}, Codec = {:?}", i + 1,
                                                 video[i].1.get_extension(),
                                                 video[i].1.get_height(),
                                                 video[i].1.get_fps(),
                                                 video[i].1.get_video_codec());
                                    }

                                    let number = read_number();
                                    if (1..=(video.len() as i32)).contains(&number) {
                                        let number = number as usize - 1;
                                        print!("Enter file name:");
                                        stdout().flush().unwrap();
                                        let mut name = String::new();
                                        stdin().read_line(&mut name).unwrap();
                                        let filename = name.trim().to_owned()
                                            + "."
                                            + video[number].1.get_extension();
                                        let err = download_file(&video[number].0, &filename).await;
                                        if err.is_err() {
                                            println!("Error occurred during downloading");
                                        }
                                    } else {
                                        println!("Not again!");
                                    }
                                }
                                2 => {
                                    if audio.is_empty() {
                                        exit(0);
                                    }
                                    println!(
                                        "Choose which file do you want to download (1-{}):",
                                        audio.len()
                                    );
                                    for i in 0..audio.len() {
                                        println!(
                                            "{}) Format = {}, AudioBitrate = {}, Codec = {:?}",
                                            i + 1,
                                            audio[i].1.get_extension(),
                                            audio[i].1.get_audio_bitrate(),
                                            audio[i].1.get_audio_codec()
                                        );
                                    }
                                    let number = read_number();
                                    if (1..=(audio.len() as i32)).contains(&number) {
                                        let number = number as usize - 1;
                                        print!("Enter file name:");
                                        stdout().flush().unwrap();
                                        let mut name = String::new();
                                        stdin().read_line(&mut name).unwrap();
                                        let filename = name.trim().to_owned()
                                            + "."
                                            + audio[number].1.get_extension();
                                        let err = download_file(&audio[number].0, &filename).await;
                                        if err.is_err() {
                                            println!("Error occurred during downloading");
                                        }
                                    } else {
                                        println!("Not again!");
                                    }
                                }
                                _ => unreachable!(),
                            }
                        } else {
                            println!("1 or 2 you, monkey");
                        }
                    }
                }
            } else {
                println!("It's not a YouTube video link!");
            }
        }
    }
}
