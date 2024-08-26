use clap::{crate_name, Arg, ArgAction, Command};
use image::{GrayAlphaImage, ImageReader, RgbaImage};

const HALF_BOTTOM: char = '▄';
const HALF_TOP: char = '▀';
const FULL_BLOCK: char = '█';

trait Image {
    fn get_alpha(&self, x: u32, y: u32) -> u8;
    fn get_color(&self, x: u32, y: u32) -> Option<(u8, u8, u8)>;

    fn width(&self) -> u32;
    fn height(&self) -> u32;
}

impl Image for GrayAlphaImage {
    fn get_alpha(&self, x: u32, y: u32) -> u8 {
        self[(x, y)].0[1]
    }
    fn get_color(&self, _: u32, _: u32) -> Option<(u8, u8, u8)> {
        None
    }

    fn width(&self) -> u32 {
        self.width()
    }
    fn height(&self) -> u32 {
        self.height()
    }
}

impl Image for RgbaImage {
    fn get_alpha(&self, x: u32, y: u32) -> u8 {
        self[(x, y)].0[3]
    }
    fn get_color(&self, x: u32, y: u32) -> Option<(u8, u8, u8)> {
        let pixel = self[(x, y)];
        Some((pixel[0], pixel[1], pixel[2]))
    }

    fn width(&self) -> u32 {
        self.width()
    }
    fn height(&self) -> u32 {
        self.height()
    }
}

fn main() {
    let cmd = Command::new(crate_name!())
        .arg(
            Arg::new("file")
                .required(true)
                .help("Relative path to file for conversion."),
        )
        .arg(
            Arg::new("grayscale")
                .long("grayscale")
                .short('g')
                .help("Don't color the output.")
                .action(ArgAction::SetTrue),
        )
        .arg_required_else_help(true);
    let matches = cmd.get_matches();

    let path: &String = matches.get_one("file").expect("file is required.");
    let Ok(image) = ImageReader::open(path) else {
        println!("error: File does not exist.");
        return;
    };

    let Ok(image) = image.decode() else {
        println!("error: Could not read image.");
        return;
    };

    let use_color = !matches.get_flag("grayscale");
    let image = if use_color {
        Box::new(image.into_rgba8()) as Box<dyn Image>
    } else {
        Box::new(image.into_luma_alpha8()) as Box<dyn Image>
    };

    for y in (0..image.height()).step_by(2) {
        for x in 0..image.width() {
            let top = image.get_alpha(x, y) == 255;
            let bottom = if y == image.height() - 1 {
                false
            } else {
                image.get_alpha(x, y + 1) == 255
            };

            let (code, reset) = if use_color {
                let (fg, bg) = match (top, bottom) {
                    (false, false) => (None, None),
                    (false, true) => (image.get_color(x, y + 1), None),
                    (true, false) => (image.get_color(x, y), None),
                    (true, true) => (image.get_color(x, y), image.get_color(x, y + 1)),
                };

                if let Some(fg) = fg {
                    if let Some(bg) = bg {
                        (
                            format!(
                                "\x1b[38;2;{fg_r};{fg_g};{fg_b};48;2;{bg_r};{bg_g};{bg_b}m",
                                fg_r = fg.0,
                                fg_g = fg.1,
                                fg_b = fg.2,
                                bg_r = bg.0,
                                bg_g = bg.1,
                                bg_b = bg.2
                            ),
                            "\x1b[0m",
                        )
                    } else {
                        (
                            format!("\x1b[38;2;{r};{g};{b}m", r = fg.0, g = fg.1, b = fg.2),
                            "\x1b[0m",
                        )
                    }
                } else {
                    (String::new(), "")
                }
            } else {
                (String::new(), "")
            };

            let character = match (top, bottom) {
                (false, false) => ' ',
                (false, true) => HALF_BOTTOM,
                (true, false) => HALF_TOP,
                (true, true) => {
                    if use_color {
                        HALF_TOP
                    } else {
                        FULL_BLOCK
                    }
                }
            };

            print!("{code}{character}{reset}")
        }

        print!("\n")
    }
}
