mod audio;

use audio::AudioPlayer;

use mizu_core::{GameBoy, GameboyConfig, JoypadButton};

use sfml::{
    graphics::{Color, FloatRect, Image, RenderTarget, RenderWindow, Sprite, Texture, View},
    system::Vector2f,
    window::{Event, Key, Style},
};

use clap::{App, Arg};

const TV_WIDTH: u32 = 160;
const TV_HEIGHT: u32 = 144;
const DEFAULT_SCALE: u32 = 5;
const DEFAULT_FPS: u32 = 60;

struct GameboyFront {
    gameboy: GameBoy,
    window: RenderWindow,
    fps: u32,
    audio_player: AudioPlayer,
    pixels_buffer: [u8; TV_HEIGHT as usize * TV_WIDTH as usize * 4],
}

impl GameboyFront {
    fn new(gameboy: GameBoy, fps: u32, scale: u32) -> Self {
        let window = RenderWindow::new(
            (TV_WIDTH * scale, TV_HEIGHT * scale),
            "",
            Style::CLOSE | Style::RESIZE,
            &Default::default(),
        );

        let audio_player = AudioPlayer::new(44100);
        audio_player.play();

        let pixels_buffer = [0xFF; TV_HEIGHT as usize * TV_WIDTH as usize * 4];

        let mut s = Self {
            gameboy,
            fps,
            window,
            audio_player,
            pixels_buffer,
        };

        // to scale the view into the window
        // this view is in the size of the GB TV screen
        // but we can scale the window and all the pixels will be scaled
        // accordingly
        s.update_view(s.window.size().x, s.window.size().y);
        s.update_fps();
        s
    }

    fn run_loop(&mut self) {
        let mut texture = Texture::new(TV_WIDTH, TV_HEIGHT).expect("texture");
        let mut t = std::time::Instant::now();

        loop {
            self.window.set_title(&format!(
                "mizu - {} - FPS: {}",
                self.gameboy.game_title(),
                (1. / t.elapsed().as_secs_f64()).round()
            ));

            t = std::time::Instant::now();

            if self.handle_key_inputs() {
                // break the loop and exit the application
                break;
            }

            self.gameboy.clock_for_frame();

            let buffer = self.gameboy.audio_buffer();

            self.audio_player.queue(&buffer);

            self.window.clear(Color::BLACK);

            convert_to_rgba(&self.gameboy.screen_buffer(), &mut self.pixels_buffer);

            let image =
                Image::create_from_pixels(TV_WIDTH, TV_HEIGHT, &self.pixels_buffer).expect("image");

            texture.update_from_image(&image, 0, 0);

            self.window.draw(&Sprite::with_texture(&texture));

            self.window.display();
        }
    }

    /// returns `true` if the app should close
    fn handle_key_inputs(&mut self) -> bool {
        while let Some(event) = self.window.poll_event() {
            match event {
                Event::Closed => {
                    return true;
                }
                Event::KeyPressed { code: key, .. } => match key {
                    Key::J => self.gameboy.press_joypad(JoypadButton::B),
                    Key::K => self.gameboy.press_joypad(JoypadButton::A),
                    Key::U => self.gameboy.press_joypad(JoypadButton::Select),
                    Key::I => self.gameboy.press_joypad(JoypadButton::Start),
                    Key::W => self.gameboy.press_joypad(JoypadButton::Up),
                    Key::S => self.gameboy.press_joypad(JoypadButton::Down),
                    Key::A => self.gameboy.press_joypad(JoypadButton::Left),
                    Key::D => self.gameboy.press_joypad(JoypadButton::Right),

                    Key::Return => {
                        self.gameboy.press_joypad(JoypadButton::A);
                        self.gameboy.press_joypad(JoypadButton::B);
                        self.gameboy.press_joypad(JoypadButton::Start);
                        self.gameboy.press_joypad(JoypadButton::Select);
                    }

                    // change FPS
                    Key::Equal => {
                        self.fps += 5;
                        self.update_fps();
                    }
                    Key::Dash => {
                        self.fps -= 5;
                        self.update_fps();
                    }
                    _ => {}
                },
                Event::KeyReleased { code: key, .. } => match key {
                    Key::J => self.gameboy.release_joypad(JoypadButton::B),
                    Key::K => self.gameboy.release_joypad(JoypadButton::A),
                    Key::U => self.gameboy.release_joypad(JoypadButton::Select),
                    Key::I => self.gameboy.release_joypad(JoypadButton::Start),
                    Key::W => self.gameboy.release_joypad(JoypadButton::Up),
                    Key::S => self.gameboy.release_joypad(JoypadButton::Down),
                    Key::A => self.gameboy.release_joypad(JoypadButton::Left),
                    Key::D => self.gameboy.release_joypad(JoypadButton::Right),

                    Key::Return => {
                        self.gameboy.release_joypad(JoypadButton::A);
                        self.gameboy.release_joypad(JoypadButton::B);
                        self.gameboy.release_joypad(JoypadButton::Start);
                        self.gameboy.release_joypad(JoypadButton::Select);
                    }
                    _ => {}
                },
                Event::Resized { width, height } => self.update_view(width, height),
                _ => {}
            }
        }

        false
    }

    fn update_view(&mut self, window_width: u32, window_height: u32) {
        let target_width = TV_WIDTH as u32;
        let target_height = TV_HEIGHT as u32;

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

        self.window.set_view(&view);
    }

    fn update_fps(&mut self) {
        self.window.set_framerate_limit(self.fps);
    }
}

fn main() {
    let default_scale_str = format!("{}", DEFAULT_SCALE);
    let default_fps_str = format!("{}", DEFAULT_FPS);

    let matches = App::new("mizu")
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
        .arg(
            Arg::with_name("fps")
                .long("fps")
                .short("f")
                .default_value(&default_fps_str)
                .takes_value(true).
                help("Specify the starting emulation speed in FPS, 0 for unlimited"),
        )
        .get_matches();

    let is_dmg = matches.is_present("dmg");
    let rom_file = matches.value_of("rom").expect("rom file argument");
    let boot_rom_file = matches.value_of("boot_rom");
    let scale = matches.value_of("scale");
    let fps = matches.value_of("fps");

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

    let fps = fps
        .and_then(|s| {
            let s = s.parse::<u32>().ok();
            if s.is_none() {
                eprintln!(
                    "[WARN] FPS must be a positive integer, using default value ({})...",
                    DEFAULT_FPS
                )
            }
            s
        })
        .unwrap_or(DEFAULT_FPS);

    let config = GameboyConfig { is_dmg };

    let gameboy = GameBoy::new(rom_file, boot_rom_file, config).unwrap();

    let mut gameboy_front = GameboyFront::new(gameboy, fps, scale);

    gameboy_front.run_loop();
}

fn convert_to_rgba(data: &[u8], output: &mut [u8]) {
    for (dest, src) in output.chunks_mut(4).zip(data.chunks(3)) {
        dest[0] = src[0];
        dest[1] = src[1];
        dest[2] = src[2];
        dest[3] = 0xFF;
    }
}
