mod audio;
use audio::AudioPlayer;

use gb_emu_core::{GameBoy, GameboyConfig, JoypadButton};

use sfml::{
    graphics::{Color, FloatRect, Image, RenderTarget, RenderWindow, Sprite, Texture, View},
    system::{SfBox, Vector2f},
    window::{Event, Key, Style},
};

use clap::{App, Arg};

const TV_WIDTH: u32 = 160;
const TV_HEIGHT: u32 = 144;
const DEFAULT_SCALE: u32 = 5;

fn get_view(
    window_width: u32,
    window_height: u32,
    target_width: u32,
    target_height: u32,
) -> SfBox<View> {
    let mut viewport = FloatRect::new(0., 0., 1., 1.);

    let screen_width = window_width as f32 / target_width as f32;
    let screen_height = window_height as f32 / target_height as f32;

    if screen_width > screen_height {
        viewport.width = screen_height / screen_width;
        viewport.left = (1. - viewport.width) / 2.;
    } else if screen_height > screen_width {
        viewport.height = screen_width / screen_height;
        viewport.top = (1. - viewport.height) / 2.;
    }

    let mut view = View::new(
        Vector2f::new((TV_WIDTH / 2) as f32, (TV_HEIGHT / 2) as f32),
        Vector2f::new((TV_WIDTH) as f32, (TV_HEIGHT) as f32),
    );

    view.set_viewport(&viewport);

    view
}

fn main() {
    let default_scale_str = format!("{}", DEFAULT_SCALE);

    let matches = App::new("GB-emu")
        .version("1.0")
        .author("Amjad Alsharafi")
        .about("Gameboy DMG and Gameboy Color emulator")
        .arg(Arg::with_name("rom").required(true))
        .arg(Arg::with_name("boot_rom"))
        .arg(
            Arg::with_name("dmg")
                .long("dmg")
                .short("d")
                .help("Operate the emulator in DMG mode"),
        )
        .arg(
            Arg::with_name("scale")
                .long("scale")
                .short("s")
                .default_value(&default_scale_str)
                .takes_value(true).
                help("Specify the amount to scale the initial display from the gameboy size of 160x144"),
        )
        .get_matches();

    let is_dmg = matches.is_present("dmg");
    let rom_file = matches.value_of("rom").expect("rom file argument");
    let boot_rom_file = matches.value_of("boot_rom");
    let scale = matches.value_of("scale");

    let config = GameboyConfig { is_dmg };

    let mut gameboy = GameBoy::new(rom_file, boot_rom_file, config).unwrap();

    let mut audio_player = AudioPlayer::new(44100);
    audio_player.play();

    let scale = scale
        .and_then(|s| {
            let s = s.parse::<u32>().ok();
            if s.is_none() {
                eprintln!(
                    "[WARN] scale must be a positive integer, using default value ({})...",
                    DEFAULT_SCALE
                )
            }
            s
        })
        .unwrap_or(DEFAULT_SCALE);

    let mut window = RenderWindow::new(
        (TV_WIDTH * scale, TV_HEIGHT * scale),
        "",
        Style::CLOSE | Style::RESIZE,
        &Default::default(),
    );

    let mut pixels_buffer = [0xFF; TV_HEIGHT as usize * TV_WIDTH as usize * 4];

    let mut fps = 60;

    window.set_framerate_limit(fps);

    // to scale the view into the window
    // this view is in the size of the GB TV screen
    // but we can scale the window and all the pixels will be scaled
    // accordingly
    window.set_view(&get_view(
        window.size().x,
        window.size().y,
        TV_WIDTH as u32,
        TV_HEIGHT as u32,
    ));

    let mut texture = Texture::new(TV_WIDTH, TV_HEIGHT).expect("texture");
    let mut t = std::time::Instant::now();

    'main: loop {
        window.set_title(&format!(
            "GB-emu - {} - FPS: {}",
            gameboy.game_title(),
            (1. / t.elapsed().as_secs_f64()).round()
        ));

        t = std::time::Instant::now();

        while let Some(event) = window.poll_event() {
            match event {
                Event::Closed => break 'main,
                Event::KeyPressed { code: key, .. } => match key {
                    Key::J => gameboy.press_joypad(JoypadButton::B),
                    Key::K => gameboy.press_joypad(JoypadButton::A),
                    Key::U => gameboy.press_joypad(JoypadButton::Select),
                    Key::I => gameboy.press_joypad(JoypadButton::Start),
                    Key::W => gameboy.press_joypad(JoypadButton::Up),
                    Key::S => gameboy.press_joypad(JoypadButton::Down),
                    Key::A => gameboy.press_joypad(JoypadButton::Left),
                    Key::D => gameboy.press_joypad(JoypadButton::Right),

                    Key::Return => {
                        gameboy.press_joypad(JoypadButton::A);
                        gameboy.press_joypad(JoypadButton::B);
                        gameboy.press_joypad(JoypadButton::Start);
                        gameboy.press_joypad(JoypadButton::Select);
                    }

                    // change FPS
                    Key::Equal => {
                        fps += 5;
                        window.set_framerate_limit(fps);
                    }
                    Key::Dash => {
                        fps -= 5;
                        window.set_framerate_limit(fps);
                    }
                    _ => {}
                },
                Event::KeyReleased { code: key, .. } => match key {
                    Key::J => gameboy.release_joypad(JoypadButton::B),
                    Key::K => gameboy.release_joypad(JoypadButton::A),
                    Key::U => gameboy.release_joypad(JoypadButton::Select),
                    Key::I => gameboy.release_joypad(JoypadButton::Start),
                    Key::W => gameboy.release_joypad(JoypadButton::Up),
                    Key::S => gameboy.release_joypad(JoypadButton::Down),
                    Key::A => gameboy.release_joypad(JoypadButton::Left),
                    Key::D => gameboy.release_joypad(JoypadButton::Right),

                    Key::Return => {
                        gameboy.release_joypad(JoypadButton::A);
                        gameboy.release_joypad(JoypadButton::B);
                        gameboy.release_joypad(JoypadButton::Start);
                        gameboy.release_joypad(JoypadButton::Select);
                    }
                    _ => {}
                },
                Event::Resized { width, height } => {
                    window.set_view(&get_view(width, height, TV_WIDTH as u32, TV_HEIGHT as u32));
                }
                _ => {}
            }
        }

        gameboy.clock_for_frame();

        let buffer = gameboy.audio_buffer();

        audio_player.queue(&buffer);

        window.clear(Color::BLACK);

        convert_to_rgba(&gameboy.screen_buffer(), &mut pixels_buffer);

        let image = Image::create_from_pixels(TV_WIDTH, TV_HEIGHT, &pixels_buffer).expect("image");

        texture.update_from_image(&image, 0, 0);

        window.draw(&Sprite::with_texture(&texture));

        window.display();
    }
}

fn convert_to_rgba(data: &[u8], output: &mut [u8]) {
    for (dest, src) in output.chunks_mut(4).zip(data.chunks(3)) {
        dest[0] = src[0];
        dest[1] = src[1];
        dest[2] = src[2];
        dest[3] = 0xFF;
    }
}
