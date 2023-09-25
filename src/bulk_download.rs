use anyhow::Result;
use dialoguer::console::Term;
use dialoguer::theme::ColorfulTheme;
use dialoguer::{Input, Select};
use home::home_dir;
use std::path::Path;
use std::process::Command;
use std::thread;

#[derive(PartialEq, Clone, Copy)]
pub enum DownloadType {
    Video,
    Audio,
}

pub fn ask_dl_type() -> Result<DownloadType> {
    let type_choices = vec!["Audio", "Video"];
    let dl_type = Select::with_theme(&ColorfulTheme::default())
        .with_prompt("Which type do you want ?")
        .items(&type_choices)
        .default(0)
        .interact_on_opt(&Term::stderr())?
        .expect("Choose something");

    match dl_type {
        0 => Ok(DownloadType::Audio),
        _ => Ok(DownloadType::Video),
    }
}

pub fn bulk_download(dl_type: DownloadType) -> Result<()> {
    let mut format = String::new();
    if dl_type == DownloadType::Video {
        let quality_choices = vec!["360p", "480p", "720p", "1080p"];
        let quality = Select::with_theme(&ColorfulTheme::default())
            .with_prompt("Which quality do you want ? (This format will be used for each video)")
            .items(&quality_choices)
            .default(0)
            .interact_on_opt(&Term::stderr())?
            .expect("Choose something");
        format = format!(
            "best[height<={}]",
            quality_choices[quality].replace("p", "")
        );
    }

    let default_location = Path::new(&home_dir().unwrap())
        .join("Downloads")
        .join("tmp");
    let location: String = Input::new()
        .with_prompt("Enter the location where you want to save your downloads")
        .default(default_location.to_str().unwrap().to_string())
        .interact_text()?;

    let urls_bulk: String = Input::new()
        .with_prompt("Enter the urls of the videos (separated by a comma)")
        .interact_text()?;

    let urls: Vec<String> = urls_bulk.split(",").map(|e| e.trim().to_string()).collect();

    let mut handles = Vec::new();
    for url in urls {
        let loc = location.clone();
        let fmt = format.clone();

        let handle = thread::spawn(move || {
            let mut command = Command::new("yt-dlp");
            command
                .current_dir(loc)
                .arg("-O")
                .arg("title")
                .arg("--no-simulate");

            match dl_type {
                DownloadType::Video => {
                    command.arg("-f").arg(fmt).arg(&url);
                }
                DownloadType::Audio => {
                    command.arg("-x").arg("--audio-format").arg("mp3").arg(&url);
                }
            }

            let out = command.output().expect("Failed to exec process.");

            (out, url)
        });
        handles.push(handle);
    }

    for handle in handles {
        let val = handle.join().expect("Thread panicked.");
        let out = val.0;
        let url = val.1;

        match out.status {
            code if code.success() => {
                println!(
                    "{}: '{}'",
                    console::style("Downloaded successfully").green().bold(),
                    String::from_utf8_lossy(&out.stdout).trim()
                )
            }
            _ => {
                println!(
                    "{}: '{}'\n{}",
                    console::style("Failed to download").red().bold(),
                    url,
                    String::from_utf8_lossy(&out.stderr)
                );
            }
        }
    }

    Ok(())
}
