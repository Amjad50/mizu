mod audio;
use audio::AudioPlayer;

use gb_emu_core::{GameBoy, JoypadButton};
use std::env::args;

use sfml::{
    graphics::{Color, FloatRect, Image, RenderTarget, RenderWindow, Sprite, Texture, View},
    system::{SfBox, Vector2f},
    window::{Event, Key, Style},
};

const TV_WIDTH: u32 = 160;
const TV_HEIGHT: u32 = 144;

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
    let args = args().collect::<Vec<String>>();

    if args.len() < 2 {
        eprintln!("USAGE: {} <rom-file> <boot-rom-file>", args[0]);
        return;
    }

    let mut gameboy = GameBoy::new(&args[1], args.get(2)).unwrap();

    let mut audio_player = AudioPlayer::new(44100);
    audio_player.play();

    let mut window = RenderWindow::new(
        (TV_WIDTH * 5, TV_HEIGHT * 5),
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
            "GB-emu - FPS: {}",
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
        dest[0] = (src[0] as f32 * 8.2) as u8;
        dest[1] = (src[1] as f32 * 8.2) as u8;
        dest[2] = (src[2] as f32 * 8.2) as u8;
        dest[3] = 0xFF;
    }
}
