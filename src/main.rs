use anyhow::Result;
use crossterm::terminal::ClearType;
use std::ffi::OsString;
use std::io::{stdout, Write};
use std::path::PathBuf;

use crossterm::{
    cursor,
    event::{read, Event, KeyCode},
    queue, style, terminal,
};

fn write_config(
    alacritty_config_base: &PathBuf,
    scheme_file: &PathBuf,
    config_out_file: &PathBuf,
) -> Result<()> {
    let mut base_content = std::fs::read_to_string(alacritty_config_base)?;
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
                Some(en.unwrap().file_name())
            } else {
                None
            }
        })
        .collect::<Vec<OsString>>())
}

fn main() -> Result<()> {
    let mut base_config = PathBuf::from("/Users/polizz/.config/alacritty/");

    let mut themes_dir = base_config.clone();
    let mut alacritty_config = base_config.clone();
    let mut curr_theme = base_config.clone();

    base_config.push("base.yml");
    themes_dir.push("themes");
    alacritty_config.push("alacritty.yml");
    curr_theme.push("curr_color");

    let current_theme = read_current(&curr_theme)?;

    let mut options = get_color_schemes(&themes_dir)?;
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
                        themes_dir.push(trimmed_theme);
                        queue!(
                            stdout,
                            style::Print("Reverting to "),
                            style::Print(trimmed_theme),
                            cursor::MoveToNextLine(1),
                        )?;
                        write_config(&base_config, &themes_dir, &alacritty_config)?;
                        break;
                    }
                    'n' => {
                        let theme = themes.next().unwrap().to_str().unwrap();
                        themes_dir.push(theme);
                        write_config(&base_config, &themes_dir, &alacritty_config)?;
                        themes_dir.pop();
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
