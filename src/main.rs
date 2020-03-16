use clap::{App, Arg};
use image::gif::GifDecoder;
use image::{DynamicImage, AnimationDecoder, GenericImage, ImageBuffer};
use hsl::HSL;
use indicatif::{ProgressBar, ProgressStyle};
use crypto::digest::Digest;
use crypto::sha3::Sha3;
use serde_json::json;
use std::fs;
use std::fs::create_dir_all;
use std::io::Write;
use std::path::Path;
use std::fmt::Debug;
use uuid::Uuid;

const IDENTICON_COLOR_BYTES: u8 = 7;
const COLORS: usize = 2;
const IDENTICON_ROWS: u8 = 5;
const ACTIVE_COLS: u8 = (IDENTICON_ROWS + 1)/2;
const HASH_MIN_LEN: u8 = ACTIVE_COLS*IDENTICON_ROWS + (COLORS as u8)*IDENTICON_COLOR_BYTES;

#[derive(Debug)]
pub enum ErrorConvert {
    HashTooShort,
    InvalidSize
}

fn hsl2rgb(h: f64, s: f64, l: f64) -> [u8; 3] {
    let rgb = (HSL { h: 360_f64 * h as f64, s: s, l: l }).to_rgb();
    [rgb.0, rgb.1, rgb.2]
}

fn normalize(value: u64, bytes: u8) -> f64 {
    value as f64 / ((1_i64 << (8 * (bytes - 1))) as  f64) // normalize to 0.0 ... 1.0
}

fn bytes_to_color(bytes: &[u8]) -> f64 {

    if bytes.len() == IDENTICON_COLOR_BYTES as usize {
        // get foreground color
        let mut fg_hue: u64 = bytes[0] as u64;

        // convert the last bytes to an uint
        for x in 1..(IDENTICON_COLOR_BYTES as usize - 1) {
            fg_hue = fg_hue << 8;
            fg_hue += bytes[x] as u64;
        }

        return normalize(fg_hue, IDENTICON_COLOR_BYTES)
    }
    0.0
}

pub fn pk_to_image(hash: &[u8], size_factor: u16) -> Result<DynamicImage, ErrorConvert> {
    if hash.len() < HASH_MIN_LEN as usize {
        return Err(ErrorConvert::HashTooShort)
    }

    if size_factor < 1 {
        return Err(ErrorConvert::InvalidSize)
    }

    // length of one image side in pixels, must be divisible by 8
    let img_side: u32 = IDENTICON_ROWS as u32 * size_factor as u32;

    let mut colors: [[u8; 3]; COLORS] = [[0, 0, 0]; COLORS];

    for color_index in 0..COLORS
        {
            let hash_part = &hash[hash.len() - (color_index + 1) * IDENTICON_COLOR_BYTES as usize.. (hash.len() - color_index * IDENTICON_COLOR_BYTES as usize)];

            let hue = bytes_to_color(hash_part);
            let lig = (color_index as f64)*0.5 + 0.3;
            let sat = 0.5;
            colors[color_index] = hsl2rgb(hue, sat, lig);
        }


    let mut color_map = [[&colors[0]; ACTIVE_COLS as usize]; IDENTICON_ROWS as usize];

    for x in 0..(IDENTICON_ROWS * ACTIVE_COLS) as usize
        {
            let row = x % (IDENTICON_ROWS as usize);
            let col = x / (IDENTICON_ROWS as usize);
            let col_index = (hash[x] as usize % COLORS) as usize;

            color_map[row][col] = &colors[col_index];
        }


    let mut img = ImageBuffer::new(img_side as u32, img_side as u32);

    //println!("{:?}", color_map);

    // draw a picture from the color_map
    for (x, y, pixel) in img.enumerate_pixels_mut() {
        let row: usize = (y / (size_factor as u32)) as usize;
        let col_tm: usize = (x / (size_factor as u32)) as usize;
        let col: usize = ((col_tm as isize *2 - (IDENTICON_ROWS as isize - 1))/2).abs() as usize; // mirror on vertical axis

        *pixel = image::Rgb(*color_map[row][col]);
    }
    Ok(DynamicImage::ImageRgb8(img))
}

fn main() {

    println!(
        "{}",
        r#"    ___               _
   /   | ____ ___  __(_)   _____  _____
  / /| |/ __ `/ / / / / | / / _ \/ ___/
 / ___ / /_/ / /_/ / /| |/ /  __/ /
/_/  |_\__, /\__,_/_/ |___/\___/_/
         /_/
"#
    );
    let matches = App::new("Aquiver")
        .version("3.0")
        .author("CAIMEO")
        .about("Playing video on Minecraft Bedrock!")
        .arg(
            Arg::with_name("path")
                .short("p")
                .long("path")
                .takes_value(true)
                .help("The path of the video(GIF)"),
        )
        .arg(
            Arg::with_name("name")
                .short("n")
                .long("name")
                .takes_value(true)
                .help("Resource pack's name(String)"),
        )
        .arg(
            Arg::with_name("description")
                .short("d")
                .long("description")
                .takes_value(true)
                .help("Descriptions"),
        )
        .arg(
            Arg::with_name("width")
                .short("w")
                .long("width")
                .takes_value(true)
                .help("The video's width (float)"),
        )
        .arg(
            Arg::with_name("height")
                .short("h")
                .long("height")
                .takes_value(true)
                .help("The video's height (float)"),
        )
        .arg(
            Arg::with_name("mode")
                .short("m")
                .long("mode")
                .takes_value(true)
                .help("Face camera mode (look_xyz, rotate_xyz etc.)"),
        )
        .arg(
            Arg::with_name("loop")
                .short("l")
                .long("loop")
                .takes_value(true)
                .help("Automatically replay the video")
        )
        .get_matches();

    let replay = matches.value_of("loop").unwrap_or("true");
    let mut auto_replay = true;
    match replay {
        "true" => {}
        _ => {
            auto_replay = false;
        }
    }
    if let Some(path) = matches.value_of("path") {
        if let Some(name) = matches.value_of("name") {
            let mut hasher = Sha3::sha3_256();
            hasher.input_str(name);
            let icon = pk_to_image(hasher.result_str().as_bytes(), 128).unwrap();

            let description = matches
                .value_of("description")
                .unwrap_or("Powered by Aquiver.");
            let height = matches
                .value_of("height")
                .unwrap_or("1")
                .parse::<f32>()
                .unwrap_or(1.0);
            let width = matches
                .value_of("width")
                .unwrap_or("2")
                .parse::<f32>()
                .unwrap_or(2.0);
            create_dir_all(Path::new(&format!("{}/behavior_pack/functions/", name)));
            create_dir_all(Path::new(&format!("{}/resource_pack/textures/frames", name)));
            create_dir_all(Path::new(&format!("{}/resource_pack/particles/frames", name)));
            icon.save(&Path::new(&format!("{}/resource_pack/pack_icon.png", name)));
            icon.save(&Path::new(&format!("{}/behavior_pack/pack_icon.png", name)));
            let manifest_res = json!({
                "format_version": 1,
                "header": {
                    "description": description,
                    "name": name,
                    "uuid": Uuid::new_v4().to_hyphenated().to_string(),
                    "version": [1, 0, 0]
                },
                "modules": [{
                    "description": description,
                    "type": "resources",
                    "uuid": Uuid::new_v4().to_hyphenated().to_string(),
                    "version": [1, 0, 0]
                }]
            });
            let manifest_dat = json!({
                "format_version": 1,
                "header": {
                    "description": description,
                    "name": name,
                    "uuid": Uuid::new_v4().to_hyphenated().to_string(),
                    "version": [1, 0, 0]
                },
                "modules": [{
                    "description": description,
                    "type": "data",
                    "uuid": Uuid::new_v4().to_hyphenated().to_string(),
                    "version": [1, 0, 0]
                }]
            });
            let face_camera_mode = matches.value_of("mode").unwrap_or("lookat_xyz");
            let mut res =
                fs::File::create(Path::new(&format!("{}/resource_pack/manifest.json", name))).unwrap();
            let mut dat =
                fs::File::create(Path::new(&format!("{}/behavior_pack/manifest.json", name))).unwrap();
            res.write_all(manifest_res.to_string().as_ref());
            dat.write_all(manifest_dat.to_string().as_ref());
            let video = fs::File::open(&Path::new(path));
            let init = vec![
                format!("scoreboard objectives remove {}", name),
                format!("scoreboard objectives add {} dummy {}", name, name),
                format!("scoreboard players add @p {} 0", name),
            ];
            let mut looping: Vec<String> = vec![];
            match video {
                Ok(img) => {
                    print!("Loading GIF Decoder\n");
                    let decoder =
                        GifDecoder::new(img).unwrap_or_else(|_| panic!("Unable to create Decoder"));
                    print!("Converting image into frames\n");
                    let frames = decoder.into_frames();
                    let frames = frames.collect_frames().expect("Error decoding image");
                    println!("Image loaded. Frames: {}", frames.len());
                    let bar = ProgressBar::new(frames.len() as u64);
                    bar.set_style(
                        ProgressStyle::default_bar()
                            .template(
                                "[{percent}%] [{bar:40.cyan/blue}] {pos:>7}/{len:7} Eta: {eta}",
                            )
                            .progress_chars("++="),
                    );

                    for (i, f) in frames.iter().enumerate() {
                        looping.push(format!("execute @a[scores={{{s}={t}}}] ~ ~ ~ execute @e[type=armor_stand,name={s}] ~ ~ ~ particle {s}:img_{t} ~ ~ ~", s = name, t = i));
                        let buf = &f.to_owned().into_buffer();
                        if let Err(e) = buf.save(&Path::new(&format!(
                            "{}/resource_pack/textures/frames/img_{}.png",
                            name, i
                        ))) {
                            println!(
                                "{} {}",
                                e,
                                format!("{}/resource_pack/textures/frames/img_{}.png", name, i)
                            );
                        }
                        let mut file = fs::File::create(&format!(
                            "{}/resource_pack/particles/frames/img_{}.json",
                            name, i
                        )).unwrap();
                        let particle = json!({
                            "format_version":"1.10.0",
                            "particle_effect":{
                                "description":{
                                    "identifier":format!("{}:img_{}",name,i),
                                    "basic_render_parameters":{
                                        "material": "particles_alpha",
                                        "texture":format!("textures/frames/img_{}.png", i)
                                    }
                                },
                                "components": {
                                    "minecraft:emitter_rate_instant": {
                                        "num_particles": 1
                                    },
                                    "minecraft:emitter_lifetime_once": {
                                        "active_time": 0.05
                                    },
                                    "minecraft:emitter_shape_point": {
                                        "offset":[0,0,0],
                                        "direction":[1,0,0]
                                    },
                                    "minecraft:particle_lifetime_expression": {
                                        "max_lifetime": 0.5
                                    },
                                    "minecraft:particle_appearance_billboard":{
                                        "facing_camera_mode":face_camera_mode,
                                        "size":[width, height]
                                    }
                                }
                            }
                        });
                        file.write_all(particle.to_string().as_ref());
                        bar.inc(1);
                    }

                    looping.push(format!(
                        "execute @p[scores={{{n}=..{t}}}] ~ ~ ~ scoreboard players add @s {n} 1",
                        n = name,
                        t = frames.len()
                    ));
                    if auto_replay {
                        looping.push(format!("execute @p[scores={{{n}={t}}}] ~ ~ ~ scoreboard players set {n} 0", n = name, t = frames.len()))
                    }
                    bar.finish();
                    let mut fn_loop = fs::File::create(Path::new(&format!(
                        "{}/behavior_pack/functions/loop.mcfunction",
                        name
                    )))
                    .unwrap();
                    fn_loop.write_all(looping.join("\n").as_bytes());
                    let mut fn_init = fs::File::create(Path::new(&format!(
                        "{}/behavior_pack/functions/init.mcfunction",
                        name
                    )))
                    .unwrap();
                    fn_init.write_all(init.join("\n").as_bytes());
                    print!("Everything was done!");
                }
                Err(e) => println!("{}", e),
            }
        }
    }
}
