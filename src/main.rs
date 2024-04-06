use std::cell::RefCell;
use std::rc::Rc;
use std::{borrow::BorrowMut, path::PathBuf};

use inquire::{InquireError, Select};

// fn set_picked_scheme(input: &str, option: &&str, string_value: &str, _idx: usize) -> Option<i64> {
//     println!(
//         "Current value: input: {:?}, option: {:?}, string_value: {:?}",
//         input, option, string_value
//     );
//
//     Some(1i64)
// }
fn main() {
    let config_dir = PathBuf::from("~/.config/alacritty/");

    let options: Vec<&str> = vec![
        "_default.yml",
        "ayu-dark.yml",
        "ayu-light.yml",
        "ayu-mirage.yml",
        "catppuccin-frappe.yml",
        "catppuccin-latte.yml",
        "catppuccin-macchiato.yml",
        "catppuccin-mocha.yml",
        "dracula.yml",
        "firewatch-light.yml",
        "firewatch.yml",
        "mountaineer-grey.yml",
        "mountaineer-light.yml",
        "mountaineer.yml",
        "night-owl.yml",
        "nord.yml",
        "papercolor-light.yml",
        "pink-moon.yml",
        "seoul256-light.yml",
        "seoul256.yml",
    ];

    let select_history: Rc<RefCell<Vec<String>>> = Rc::new(RefCell::new(vec![]));

    let cloned_select_history = select_history.clone();
    let set_picked_scheme =
        |_input: &str, _option: &&str, string_value: &str, _idx: usize| -> Option<i64> {
            // println!(
            //     "Current value: input: {:?}, option: {:?}, string_value: {:?}",
            //     input, option, string_value
            // );
            let mut cloned_select_history = (*cloned_select_history).borrow_mut();
            cloned_select_history.push(string_value.to_owned());

            Some(1i64)
        };

    let scheme: Result<&str, InquireError> =
        Select::new("Select a color scheme and enter when finished:", options)
            .with_scorer(&set_picked_scheme)
            .with_vim_mode(true)
            .prompt();

    match scheme {
        Ok(choice) => println!("{}! That's mine too!", choice),
        Err(_) => println!("There was an error, please try again"),
    }

    println!("Select history: {:#?}", (*select_history).borrow_mut());
}

// config_dir=~/.config
// alacritty_config=${config_dir}/alacritty
// alacritty_themes=${alacritty_config}/themes
// curr_color_file=${alacritty_config}/curr_color
//
// declare -a arr=($(ls -al ${alacritty_themes} | awk 'NR>3 { printf("%s ", $9) }'))
//
// curr_color=$(cat ${curr_color_file} 2>/dev/null)
//
// if [[ -z "$curr_color" ]]; then
//    next_color=${arr[0]}
// else
//    len=${#arr[@]}
//    for ((i = 0; i < len; i++));
//    do
//       # echo "Checking: ${arr[$i]} against: ${curr_color}"
//       if [ "${arr[$i]}" == "$curr_color" ]; then
//          if [ $((i+1)) -ge "$len" ]; then
//             next_color="${arr[0]}"
//          else
//             next_color="${arr[$i+1]}"
//          fi
//
//          break
//       fi
//    done
// fi
//
// $(echo ${next_color} > "${curr_color_file}")
// echo "Picking next color: ${next_color}"
//
// cat ~/.config/alacritty/base.yml ~/.config/alacritty/themes/${next_color} > ~/.config/alacritty/alacritty.yml
