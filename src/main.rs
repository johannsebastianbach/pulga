mod error;
mod logos;
mod newfetch;
mod util;
mod uname;

use crate::newfetch::UserData;

use indoc::indoc;
use sugars::{boxed, hmap};
use termion::{color::*, cursor::*};

use std::collections::HashMap;

fn show(text: String, logo: &str) {
    let lines: Vec<String> = text.lines().map(|x| x.trim().to_string()).collect();

    // Code to show colored logo
    #[rustfmt::skip]
    let color_map = {
        let mut m: HashMap<char, Box<dyn Color>> = hmap! {};
        m.insert('k', boxed!(Black  )); // k => Black
        m.insert('b', boxed!(Blue   )); // b => Blue
        m.insert('c', boxed!(Cyan   )); // c => Cyan
        m.insert('g', boxed!(Green  )); // g => Green
        m.insert('m', boxed!(Magenta)); // m => Magenta
        m.insert('r', boxed!(Red    )); // r => Red
        m.insert('w', boxed!(White  )); // w => White
        m.insert('y', boxed!(Yellow )); // y => Yellow
        m.insert('R', boxed!(Reset  )); // R => Reset all
        m
    };

    let logo = logo.chars().collect::<Vec<char>>();

    let mut i = 0;
    while i < logo.len() - 2 {
        let slice = &logo[i..=i + 2];

        match slice {
            ['{', color_id, '}'] => {
                let color: &dyn Color = match color_map.get(color_id) {
                    None => panic!("Unexpected color_id '{}'", color_id),
                    Some(color) => color.as_ref(),
                };

                print!("{}", Fg(color));
                i += 3;
            },
            [first, ..] => {
                print!("{}", *first as char);
                i += 1;
            },
            _ => unreachable!(),
        }
    }

    // Show the remaining stuff
    for remaining in logo.iter().skip(i) {
        print!("{}", remaining);
    }
    println!();

    // Code to insert information at the side of the logo
    print!("{}", Up(14));

    for (_, line) in lines.iter().enumerate() {
        print!("{} {}{}{}", Right(32), line, Left(1000), Down(1));
    }
    print!("{}", Down(18));
}

fn main() {
    let data: UserData = newfetch::get_user_data();

    #[rustfmt::skip]
    let text = format!(indoc! {
        "{c}{}{w}: {r}{}{R}
         {c}{}{w}: {r}{}{R}
         {c}{}{w}: {r}{}{R}
         {c}{}{w}: {r}{}{R}
         {c}{}{w}: {r}{}{R}
         {c}{}{w}: {r}{}{R}
         {c}{}{w}: {r}{}{R}
         {c}{}{w}: {r}{}{R}
         {c}{}{w}: {r}{}{R}
         {c}{}{w}: {r}{}{R} / {r}{}{R}"
        },
        "username", data.username,
        "hostname", data.hostname,
        "cpu", data.cpu_info,
        "uptime", newfetch::get_uptime().unwrap(),
        "home", data.hmd,
        "shell", data.shell,
        "distro", data.distro,
        "kernel", data.kernel_version,
        "desktop env.", data.desk_env,
        "memory usage", data.used_memory, data.total_memory,
        c = Fg(LightCyan),
        w = Fg(LightBlack),
        R = Fg(Reset),
        r = Fg(LightRed)
    );

    // println!("{}", text.len());

    let logo = logos::choose_logo(logos::Logo::Debian);
    // println!("{}", logo);

    show(text, logo);
}
