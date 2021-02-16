mod audio;

use std::cell::RefCell;
use std::rc::Rc;

use audio::AudioPlayer;

use mizu_core::{GameBoy, GameboyConfig, JoypadButton, Printer};

use sfml::{
    graphics::{
        Color, FloatRect, Image, RenderTarget, RenderWindow, Sprite, Texture, Transformable, View,
    },
    system::{SfBox, Vector2f},
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
    printer: Option<Rc<RefCell<Printer>>>,
    printer_window: Option<RenderWindow>,
    printer_window_scroll: u32,
}

impl GameboyFront {
    fn new(gameboy: GameBoy, fps: u32, scale: u32) -> Self {
        let mut window = RenderWindow::new(
            (TV_WIDTH * scale, TV_HEIGHT * scale),
            "",
            Style::CLOSE | Style::RESIZE,
            &Default::default(),
        );
        let size = window.size();
        Self::update_window_view(&mut window, size.x, size.y);

        let audio_player = AudioPlayer::new(44100);
        audio_player.play();

        let pixels_buffer = [0xFF; TV_HEIGHT as usize * TV_WIDTH as usize * 4];

        let mut s = Self {
            gameboy,
            fps,
            window,
            audio_player,
            pixels_buffer,
            printer: None,
            printer_window: None,
            printer_window_scroll: 0,
        };

        // to scale the view into the window
        // this view is in the size of the GB TV screen
        // but we can scale the window and all the pixels will be scaled
        // accordingly
        s.update_fps();
        s
    }

    fn connect_printer(&mut self) {
        let printer = Rc::new(RefCell::new(Printer::default()));
        self.gameboy.connect_device(printer.clone());
        self.printer = Some(printer);

        let mut printer_window = RenderWindow::new(
            (TV_WIDTH * 5, TV_HEIGHT * 5),
            "",
            Style::CLOSE | Style::RESIZE,
            &Default::default(),
        );
        let size = printer_window.size();
        Self::update_window_view(&mut printer_window, size.x, size.y);

        printer_window.set_title("mizu printer");

        // do not block on update (infinite framerate)
        printer_window.set_framerate_limit(0);

        self.printer_window = Some(printer_window);
        self.printer_window_scroll = 0;
    }

    fn disconnect_printer(&mut self) {
        self.gameboy.disconnect_device();
        self.printer = None;
        // will set `printer_window` to `None`
        self.printer_window.take().unwrap().close();
        self.printer_window_scroll = 0;
    }

    fn update_printer_window(&mut self) {
        if self.handle_printer_key_inputs() {
            self.disconnect_printer();
            return;
        }

        if let Some(printer) = self.printer.as_ref() {
            let printer = printer.borrow();
            let printer_image_buffer = printer.get_image_buffer();
            let (width, height) = printer.get_image_size();

            let printer_window = self.printer_window.as_mut().unwrap();

            if width * height != 0 {
                let mut texture = Texture::new(width, height).expect("texture");
                let mut window_image_buffer = vec![0; width as usize * height as usize * 4];

                // printer will output RGB but SFML need RGBA
                convert_to_rgba(printer_image_buffer, &mut window_image_buffer);

                let image =
                    Image::create_from_pixels(width, height, &window_image_buffer).expect("image");

                texture.update_from_image(&image, 0, 0);
                let mut sprite = Sprite::with_texture(&texture);
                sprite.set_position((0., -(self.printer_window_scroll as f32)));

                printer_window.draw(&sprite);
            } else {
                printer_window.clear(Color::BLACK);
            }

            printer_window.display();
        }
    }

    fn handle_printer_key_inputs(&mut self) -> bool {
        let max_scroll = self.get_max_printer_window_scroll() as i32;

        if let Some(mut printer_window) = self.printer_window.as_mut() {
            while let Some(event) = printer_window.poll_event() {
                match event {
                    Event::Closed => {
                        return true;
                    }
                    Event::Resized { width, height } => {
                        Self::update_window_view(&mut printer_window, width, height);
                    }
                    Event::KeyPressed { code: key, .. } => match key {
                        Key::C => {
                            // clear image buffer
                            self.printer
                                .as_ref()
                                .unwrap()
                                .borrow_mut()
                                .clear_image_buffer();
                            self.printer_window_scroll = 0;
                        }
                        _ => {}
                    },
                    Event::MouseWheelScrolled { delta, .. } => {
                        // we use sub to invert
                        // speed of scroll
                        let delta = delta as i32 * 5;
                        let scroll = self.printer_window_scroll as i32;
                        let scroll = scroll - delta;
                        // if its negative, keep it zero
                        let scroll = scroll.max(0);
                        // don't exceed max scroll
                        let scroll = scroll.min(max_scroll);

                        self.printer_window_scroll = scroll as u32;
                    }
                    _ => {}
                }
            }
        }

        false
    }

    fn get_max_printer_window_scroll(&self) -> u32 {
        if let Some(printer) = self.printer.as_ref() {
            let (_w, h) = printer.borrow().get_image_size();

            // it will be zero if `TV_HEIGHT` is larger than `h`
            h.saturating_sub(TV_HEIGHT)
        } else {
            0
        }
    }

    fn run_loop(&mut self) {
        let mut texture = Texture::new(TV_WIDTH, TV_HEIGHT).expect("texture");
        let mut t = std::time::Instant::now();

        loop {
            self.window.set_title(&format!(
                "mizu - {} - FPS: {} - printer {}connected",
                self.gameboy.game_title(),
                (1. / t.elapsed().as_secs_f64()).round(),
                // the format! has "{}connected", so we just fill `"dis"` if needed
                if self.printer.is_some() { "" } else { "dis" },
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

            // if any
            self.update_printer_window();

            // frame limiting, must be last
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

                    Key::P => {
                        if self.printer.is_some() {
                            self.disconnect_printer();
                        } else {
                            self.connect_printer();
                        }
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
                Event::Resized { width, height } => {
                    Self::update_window_view(&mut self.window, width, height)
                }
                _ => {}
            }
        }

        false
    }

    fn get_new_view(
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

    fn update_window_view(window: &mut RenderWindow, window_width: u32, window_height: u32) {
        window.set_view(&Self::get_new_view(
            window_width,
            window_height,
            TV_WIDTH,
            TV_HEIGHT,
        ));
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
