use anyhow::{anyhow, Result};
use crossterm::terminal::ClearType;
use std::ffi::OsString;
use std::io::{stdout, Write};
use std::path::PathBuf;

const HELP: &str = "\
App

USAGE:
  colorschemer [OPTIONS]

FLAGS:
  -h, --help            Prints help information

OPTIONS:
  --root                STRING       DEFAULT: ~/.config/alacritty                   Config dir for profile (i.e. ~/.config/alacritty)
  --themes              STRING       DEFAULT: ~/.config/alacritty/themes            Theme sub-directory under root config 
  --base-config         STRING       DEFAULT: ~/.config/alacritty/base.toml         Base Alacritty config without color information
  --out-file            STRING       DEFAULT: ~/.config/alacritty/alacritty.toml    Destination Alacritty config file to write 
  --pick-file           STRING       OPTIONAL                                       Immediately select a theme file                                           
ARGS:
  <INPUT>
";

#[derive(Debug)]
struct AppArgs {
    root_config: PathBuf,
    themes_dir: PathBuf,
    base_config_file: String,
    out_file: String,
    pick_file: Option<String>,
}

use crossterm::{
    cursor,
    event::{read, Event, KeyCode},
    queue, style, terminal,
};

fn write_config(
    config_base_file: &PathBuf,
    scheme_file: &PathBuf,
    config_out_file: &PathBuf,
) -> Result<()> {
    let mut base_content = std::fs::read_to_string(config_base_file)?;
    let theme_content = std::fs::read_to_string(scheme_file)?;
    base_content.push_str(&theme_content);

    std::fs::write(config_out_file, base_content)?;

    Ok(())
}

fn save_current(curr_theme_save_file: &PathBuf, curr_theme: &str) -> Result<()> {
    std::fs::write(curr_theme_save_file, curr_theme).map_err(|e| e.into())
}

fn read_current(curr_file_path: &PathBuf) -> Option<String> {
    std::fs::read_to_string(curr_file_path).ok()
}

fn get_color_schemes(dir: &PathBuf) -> Result<Vec<OsString>> {
    let contents = dir.read_dir()?;

    Ok(contents
        .filter_map(|en| {
            let en = en.ok()?;
            let ft = en.file_type().ok()?;

            if ft.is_file() {
                Some(en.file_name())
            } else {
                None
            }
        })
        .collect::<Vec<OsString>>())
}

fn main() -> Result<()> {
    let args = match parse_args() {
        Ok(v) => v,
        Err(e) => {
            eprintln!("Error: {}.", e);
            std::process::exit(1);
        }
    };

    let AppArgs {
        mut root_config,
        themes_dir,
        base_config_file,
        out_file,
        pick_file,
    } = args;

    let mut themes_folder = root_config.clone();
    let mut out_file_path = root_config.clone();
    let mut curr_theme_save_file = root_config.clone();

    root_config.push(base_config_file);
    themes_folder.push(themes_dir.clone());
    out_file_path.push(out_file);
    curr_theme_save_file.push("curr_theme");

    // println!(
    //     "root: {:?}, themes: {:?}, out_file: {:?}, curr_theme: {:?}",
    //     &root_config.to_str(),
    //     &themes_folder.to_str(),
    //     &out_file_path.to_str(),
    //     &curr_theme.to_str()
    // );

    let mut stdout = stdout();
    let current_theme = read_current(&curr_theme_save_file);
    let mut last_viewed_theme: Option<&str> = None;

    if let Some(picked_theme) = pick_file {
        queue!(
            stdout,
            style::Print("Saving picked theme "),
            style::Print(&picked_theme),
            cursor::MoveToNextLine(1),
        )?;
        themes_folder.push(&picked_theme);
        write_config(&root_config, &themes_folder, &out_file_path)?;
        save_current(&curr_theme_save_file, &picked_theme)?;

        stdout.flush()?;
    } else {
        let mut options = get_color_schemes(&themes_folder)?;
        options.sort();

        terminal::enable_raw_mode()?;
        queue!(stdout, cursor::MoveToNextLine(1))?;

        let current_theme_msg = format!(
            "Current theme: {:?}",
            current_theme.as_ref().map_or("None", |c| c.trim())
        );

        queue!(
            stdout,
            style::Print(current_theme_msg),
            cursor::MoveToNextLine(0),
            style::Print("Press [n] to cycle themes, [q] to exit, [enter] to accept"),
            cursor::MoveToNextLine(0),
            style::Print("\r\n")
        )?;
        stdout.flush()?;

        let mut themes = options.iter().cycle();
        loop {
            let char = read()?;

            match char {
                Event::Key(key_event) => match key_event.code {
                    KeyCode::Enter => {
                        if let Some(last_theme) = last_viewed_theme {
                            queue!(
                                stdout,
                                style::Print("Saving theme "),
                                style::Print(last_theme),
                                cursor::MoveToNextLine(1),
                            )?;
                            save_current(&curr_theme_save_file, &last_theme)?;
                        }
                        // .ok_or(anyhow!("No new theme picked, no new theme saved"))?;

                        break;
                    }
                    KeyCode::Char(code) => match code {
                        'q' => {
                            if let Some(theme) = current_theme {
                                let trimmed_theme = theme.trim();
                                themes_folder.push(trimmed_theme);
                                queue!(
                                    stdout,
                                    style::Print("Reverting to "),
                                    style::Print(trimmed_theme),
                                    cursor::MoveToNextLine(1),
                                )?;
                                write_config(&root_config, &themes_folder, &out_file_path)?;
                            }
                            break;
                        }
                        'n' => {
                            let theme_file = themes
                                .next()
                                .ok_or(anyhow!("Should never stop cycling through themes"))?
                                .to_str()
                                .ok_or(anyhow!("Could not get theme filename"))?;
                            themes_folder.push(theme_file);
                            last_viewed_theme = Some(theme_file);
                            write_config(&root_config, &themes_folder, &out_file_path)?;
                            themes_folder.pop();
                            queue!(
                                stdout,
                                terminal::Clear(ClearType::CurrentLine),
                                style::Print(theme_file),
                                cursor::MoveToColumn(0)
                            )?;
                        }
                        _ => {}
                    },
                    _ => {}
                },
                _ => {}
            }

            stdout.flush()?;
        }
    }

    terminal::disable_raw_mode()?;

    Ok(())
}

fn parse_args() -> Result<AppArgs, pico_args::Error> {
    let mut pargs = pico_args::Arguments::from_env();

    if pargs.contains(["-h", "--help"]) {
        print!("{}", HELP);
        std::process::exit(0);
    }

    let args = AppArgs {
        root_config: pargs
            .value_from_os_str("--root", parse_path)
            .unwrap_or("~/.config/alacritty/".into()),
        themes_dir: pargs.value_from_str("--themes").unwrap_or("themes".into()),
        base_config_file: pargs
            .value_from_str("--base-config")
            .unwrap_or("base.toml".into()),
        out_file: pargs
            .value_from_str("--out-file")
            .unwrap_or("alacritty.toml".into()),
        pick_file: pargs.opt_value_from_str("--pick-file")?,
    };

    let remaining = pargs.finish();
    if !remaining.is_empty() {
        eprintln!("Warning: unused arguments left: {:?}.", remaining);
    }

    Ok(args)
}

fn parse_path(s: &std::ffi::OsStr) -> Result<std::path::PathBuf, &'static str> {
    Ok(s.into())
}
