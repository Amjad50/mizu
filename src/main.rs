mod audio;
mod notification;
mod printer_front;

use std::{
    fs::{self, File},
    io::Write,
    path::{Path, PathBuf},
};

use audio::AudioPlayer;
use directories_next::ProjectDirs;
use notification::Notifications;
use printer_front::MizuPrinter;

use mizu_core::{GameBoy, GameboyConfig, JoypadButton, SaveError};

use sfml::{
    graphics::{Color, FloatRect, Image, RenderTarget, RenderWindow, Sprite, Texture, View},
    system::Vector2f,
    window::{Event, Key, Style},
    SfBox,
};

use clap::{App, Arg};

pub const TV_WIDTH: u32 = 160;
pub const TV_HEIGHT: u32 = 144;
const DEFAULT_SCALE: u32 = 5;
const DEFAULT_FPS: u32 = 60;

struct GameboyFront {
    gameboy: GameBoy,
    window: RenderWindow,
    fps: u32,
    audio_player: AudioPlayer,
    pixels_buffer: [u8; TV_HEIGHT as usize * TV_WIDTH as usize * 4],
    printer: Option<MizuPrinter>,
    notifications: Notifications,
}

impl GameboyFront {
    fn new(gameboy: GameBoy, fps: u32, scale: u32) -> Self {
        let mut window = RenderWindow::new(
            (TV_WIDTH * scale, TV_HEIGHT * scale),
            "",
            Style::CLOSE | Style::RESIZE,
            &Default::default(),
        );
        let mut notifications = Notifications::new();

        let size = window.size();

        update_window_view(&mut window, size.x, size.y);
        notifications.update_size(size.x, size.y);

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
            notifications,
        };

        s.update_fps();
        s
    }

    fn connect_printer(&mut self) {
        let mizu_printer = MizuPrinter::default();
        self.gameboy.connect_device(mizu_printer.get_printer());
        self.printer = Some(mizu_printer);
    }

    fn disconnect_printer(&mut self) {
        self.gameboy.disconnect_device();
        self.printer.take().unwrap().close();
    }

    fn run_loop(&mut self) {
        let mut texture = Texture::new(TV_WIDTH, TV_HEIGHT).expect("texture");
        let mut t = std::time::Instant::now();

        loop {
            let elapsed = t.elapsed().as_secs_f32();
            self.window.set_title(&format!(
                "mizu - {} - FPS: {} - printer {}connected",
                self.gameboy.game_title(),
                (1. / elapsed).round(),
                // the format! has "{}connected", so we just fill `"dis"` if needed
                if self.printer.is_some() { "" } else { "dis" },
            ));
            self.notifications.update(elapsed);

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
            if let Some(printer) = self.printer.as_mut() {
                if printer.update_printer_window() {
                    self.disconnect_printer();
                }
            }

            let size = self.window.size();

            // restore normal size of the window without stretching
            self.window
                .set_view(&get_new_view(size.x, size.y, size.x, size.y));
            // draw the notifications
            self.window.draw(&self.notifications);
            // restore gameboy stretched size
            update_window_view(&mut self.window, size.x, size.y);

            // frame limiting, must be last
            self.window.display();
        }
    }

    fn base_save_state_folder(&self) -> Option<PathBuf> {
        if let Some(proj_dirs) = ProjectDirs::from("", "Amjad50", "Mizu") {
            let base_saved_states_dir = proj_dirs.data_local_dir().join("saved_states");
            // Linux:   /home/<user>/.local/share/mizu/saved_states
            // Windows: C:\Users\<user>\AppData\Local\Amjad50\Mizu\data\saved_states
            // macOS:   /Users/<user>/Library/Application Support/Amjad50.Mizu/saved_states
            fs::create_dir_all(&base_saved_states_dir).ok()?;
            Some(base_saved_states_dir)
        } else {
            None
        }
    }

    fn save_state_file(&self, slot: u8) -> Option<Box<Path>> {
        let cartridge_path = self.gameboy.file_path();

        if let Some(base_saved_states_dir) = self.base_save_state_folder() {
            // we use the cartridge path and replace all `.` with `_` to remove
            // the extensions but keep the unique filename
            let save_file_base_name = format!(
                "{}_{}",
                cartridge_path
                    .file_name()
                    .unwrap()
                    .to_string_lossy()
                    .replace('.', "_"),
                slot
            );
            Some(
                base_saved_states_dir
                    .join(save_file_base_name)
                    .with_extension("mst")
                    .into_boxed_path(),
            )
        } else {
            None
        }
    }

    fn save_state(&self, slot: u8) -> Result<(), SaveError> {
        let file_path = self.save_state_file(slot).ok_or(SaveError::SaveFileError)?;
        println!("saving state file {}", file_path.to_string_lossy());
        let mut file = File::create(file_path)?;

        // first save to a vector as writing to the file is very slow (maybe because of the flushes)
        let mut data = Vec::new();
        self.gameboy.save_state(&mut data)?;

        // write content of the saved_state to the file
        file.write_all(&data)?;

        Ok(())
    }

    /// return `true` if the file is loaded, `false` otherwize in case of no errors
    fn load_state(&mut self, slot: u8) -> Result<bool, SaveError> {
        let file_path = self.save_state_file(slot).ok_or(SaveError::SaveFileError)?;
        println!("trying to load state file {}", file_path.to_string_lossy());

        if file_path.exists() {
            let file = File::open(file_path)?;
            self.gameboy.load_state(file)?;
            Ok(true)
        } else {
            Ok(false)
        }
    }

    fn notify(&mut self, msg: &str) {
        self.notifications.add_msg(msg)
    }

    fn num_key(key: Key) -> Option<u8> {
        match key {
            Key::NUM1 => Some(0),
            Key::NUM2 => Some(1),
            Key::NUM3 => Some(2),
            Key::NUM4 => Some(3),
            Key::NUM5 => Some(4),
            Key::NUM6 => Some(5),
            Key::NUM7 => Some(6),
            Key::NUM8 => Some(7),
            Key::NUM9 => Some(8),
            Key::NUM0 => Some(9),
            _ => None,
        }
    }

    /// returns `true` if the app should close
    fn handle_key_inputs(&mut self) -> bool {
        while let Some(event) = self.window.poll_event() {
            match event {
                Event::Closed => {
                    return true;
                }
                Event::KeyPressed {
                    code: key, shift, ..
                } => match key {
                    Key::J => self.gameboy.press_joypad(JoypadButton::B),
                    Key::K => self.gameboy.press_joypad(JoypadButton::A),
                    Key::U => self.gameboy.press_joypad(JoypadButton::Select),
                    Key::I => self.gameboy.press_joypad(JoypadButton::Start),
                    Key::W => self.gameboy.press_joypad(JoypadButton::Up),
                    Key::S => self.gameboy.press_joypad(JoypadButton::Down),
                    Key::A => self.gameboy.press_joypad(JoypadButton::Left),
                    Key::D => self.gameboy.press_joypad(JoypadButton::Right),

                    _ if Self::num_key(key).is_some() && shift => {
                        let state_n = Self::num_key(key).unwrap();

                        let notification_msg = match self.load_state(state_n) {
                            Ok(false) => {
                                format!(
                                    "[Save state] save state #{} could not be found.",
                                    state_n + 1
                                )
                            }
                            Ok(true) => format!("[Save state] loaded save state #{}.", state_n + 1),
                            Err(e) => format!(
                                "[Save state] error while loading from #{} {}.",
                                state_n + 1,
                                e
                            ),
                        };

                        self.notify(&notification_msg);
                    }
                    _ if Self::num_key(key).is_some() => {
                        let state_n = Self::num_key(key).unwrap();
                        let notification_msg = match self.save_state(state_n) {
                            Ok(_) => {
                                format!("[Save state] saved save state #{}.", state_n + 1)
                            }
                            Err(e) => format!(
                                "[Save state] error while saving into #{} {}.",
                                state_n + 1,
                                e
                            ),
                        };

                        self.notify(&notification_msg);
                    }

                    Key::ENTER => {
                        self.gameboy.press_joypad(JoypadButton::A);
                        self.gameboy.press_joypad(JoypadButton::B);
                        self.gameboy.press_joypad(JoypadButton::Start);
                        self.gameboy.press_joypad(JoypadButton::Select);
                    }

                    // change FPS
                    Key::EQUAL => {
                        self.fps += 5;
                        self.update_fps();
                    }
                    Key::HYPHEN => {
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

                    Key::ENTER => {
                        self.gameboy.release_joypad(JoypadButton::A);
                        self.gameboy.release_joypad(JoypadButton::B);
                        self.gameboy.release_joypad(JoypadButton::Start);
                        self.gameboy.release_joypad(JoypadButton::Select);
                    }
                    _ => {}
                },
                Event::Resized { width, height } => {
                    update_window_view(&mut self.window, width, height);
                    self.notifications.update_size(width, height);
                }
                _ => {}
            }
        }

        false
    }

    fn update_fps(&mut self) {
        self.window.set_framerate_limit(self.fps);
    }
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
        Vector2f::new((target_width / 2) as f32, (target_height / 2) as f32),
        Vector2f::new((target_width) as f32, (target_height) as f32),
    );

    view.set_viewport(&viewport);

    view
}

/// to scale the view into the window
/// this view is in the size of the GB LCD screen
/// but we can scale the window and all the pixels will be scaled
/// accordingly
pub fn update_window_view(window: &mut dyn RenderTarget, window_width: u32, window_height: u32) {
    window.set_view(&get_new_view(
        window_width,
        window_height,
        TV_WIDTH,
        TV_HEIGHT,
    ));
}

pub fn convert_to_rgba(data: &[u8], output: &mut [u8]) {
    for (dest, src) in output.chunks_mut(4).zip(data.chunks(3)) {
        dest[0] = src[0];
        dest[1] = src[1];
        dest[2] = src[2];
        dest[3] = 0xFF;
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
