use anyhow::Result;
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
ARGS:
  <INPUT>
";

#[derive(Debug)]
struct AppArgs {
    root_config: PathBuf,
    themes_dir: PathBuf,
    base_config_file: String,
    out_file: String,
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

fn read_current(curr_file_path: &PathBuf) -> Result<String> {
    let current_theme = std::fs::read_to_string(curr_file_path).unwrap_or("".to_owned());
    Ok(current_theme)
}

fn get_color_schemes(dir: &PathBuf) -> Result<Vec<OsString>> {
    let contents = dir.read_dir()?;

    Ok(contents
        .filter_map(|en| {
            if en.is_ok() {
                let file = en.unwrap();

                if file.file_type().unwrap().is_file() {
                    Some(file.file_name())
                } else {
                    None
                }
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
    } = args;

    let mut themes_folder = root_config.clone();
    let mut out_file_path = root_config.clone();
    let mut curr_theme = root_config.clone();

    root_config.push(base_config_file);
    themes_folder.push(themes_dir.clone());
    out_file_path.push(out_file);
    curr_theme.push("curr_color");

    // println!(
    //     "root: {:?}, themes: {:?}, out_file: {:?}, curr_theme: {:?}",
    //     &root_config.to_str(),
    //     &themes_folder.to_str(),
    //     &out_file_path.to_str(),
    //     &curr_theme.to_str()
    // );

    let current_theme = read_current(&curr_theme)?;
    let mut options = get_color_schemes(&themes_folder)?;
    options.sort();

    terminal::enable_raw_mode()?;
    let mut stdout = stdout();
    queue!(stdout, cursor::MoveToNextLine(1))?;

    let current_theme_msg = format!("Current theme: {}", &current_theme);
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
                    queue!(
                        stdout,
                        style::Print("Saving theme"),
                        cursor::MoveToNextLine(1),
                    )?;

                    break;
                }
                KeyCode::Char(code) => match code {
                    'q' => {
                        let trimmed_theme = current_theme.trim();
                        themes_folder.push(trimmed_theme);
                        queue!(
                            stdout,
                            style::Print("Reverting to "),
                            style::Print(trimmed_theme),
                            cursor::MoveToNextLine(1),
                        )?;
                        write_config(&root_config, &themes_folder, &out_file_path)?;
                        break;
                    }
                    'n' => {
                        let theme = themes.next().unwrap().to_str().unwrap();
                        themes_folder.push(theme);
                        write_config(&root_config, &themes_folder, &out_file_path)?;
                        themes_folder.pop();
                        queue!(
                            stdout,
                            terminal::Clear(ClearType::CurrentLine),
                            style::Print(theme),
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
