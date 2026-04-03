use cfonts::{Fonts, Options, render};

pub fn banner(s: &String) -> String {
    render(Options {
        text: String::from(s),
        font: Fonts::FontPallet,
        ..Options::default()
    })
    .text
}

pub fn banner_small(s: &String) -> String {
    render(Options {
        text: String::from(s),
        font: Fonts::FontTiny,
        ..Options::default()
    })
    .text
}

pub fn bds_banner() -> String {
    let ban = format!("BDS {}", env!("CARGO_PKG_VERSION"));
    banner(&ban)
}
