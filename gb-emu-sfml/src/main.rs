use gb_emu_core::{GameBoy, JoypadButton};
use std::env::args;
use std::sync::Arc;

use sfml::{
    graphics::{Color, FloatRect, Image, RenderTarget, RenderWindow, Sprite, Texture, View},
    system::{SfBox, Vector2f},
    window::{Event, Key, Style},
};

const TV_WIDTH: u32 = 160;
const TV_HEIGHT: u32 = 144;

const SCREEN_WIDTH: u32 = TV_WIDTH * 3;
const SCREEN_HEIGHT: u32 = TV_HEIGHT * 3;

struct Player {
    buffer: ringbuf::Consumer<f32>,
}

impl Iterator for Player {
    type Item = f32;

    fn next(&mut self) -> Option<Self::Item> {
        Some(self.buffer.pop().unwrap_or(0.))
    }
}

impl rodio::Source for Player {
    fn current_frame_len(&self) -> Option<usize> {
        None
    }

    fn channels(&self) -> u16 {
        1
    }

    fn sample_rate(&self) -> u32 {
        22050
    }

    fn total_duration(&self) -> Option<std::time::Duration> {
        None
    }
}

fn get_player(buffer: ringbuf::Consumer<f32>) -> Option<(rodio::OutputStream, rodio::Sink)> {
    let (stream, stream_handle) = rodio::OutputStream::try_default().ok()?;

    // bug in rodio, that it panics if the device does not support any format
    // it is fixed now in github, not sure when is the release coming
    let sink = rodio::Sink::try_new(&stream_handle).ok()?;

    // let (input, output) = rodio::queue::queue::<f32>(true);

    let low_pass_player = rodio::source::Source::low_pass(Player { buffer }, 10000);

    sink.append(low_pass_player);
    sink.set_volume(0.15);

    sink.pause();

    Some((stream, sink))
}

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
        eprintln!("USAGE: {} <rom-file>", args[0]);
        return;
    }

    let mut gameboy = GameBoy::new(&args[1]).unwrap();

    let buffer = ringbuf::RingBuffer::<f32>::new(20000);
    let (mut producer, consumer) = buffer.split();

    let (_stream, sink) = get_player(consumer).unwrap();

    sink.play();

    let mut window = RenderWindow::new(
        (SCREEN_WIDTH, SCREEN_HEIGHT),
        "GB test",
        Style::CLOSE | Style::RESIZE,
        &Default::default(),
    );
    window.set_vertical_sync_enabled(true);

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

    'main: loop {
        while let Some(event) = window.poll_event() {
            match event {
                Event::Closed
                | Event::KeyPressed {
                    code: Key::Escape, ..
                } => break 'main,
                Event::KeyPressed { code: key, .. } => match key {
                    Key::J => gameboy.press_joypad(JoypadButton::B),
                    Key::K => gameboy.press_joypad(JoypadButton::A),
                    Key::U => gameboy.press_joypad(JoypadButton::Select),
                    Key::I => gameboy.press_joypad(JoypadButton::Start),
                    Key::W => gameboy.press_joypad(JoypadButton::Up),
                    Key::S => gameboy.press_joypad(JoypadButton::Down),
                    Key::A => gameboy.press_joypad(JoypadButton::Left),
                    Key::D => gameboy.press_joypad(JoypadButton::Right),
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
        producer.push_slice(&buffer);

        window.clear(Color::WHITE);

        let pixels = convert_to_rgba(gameboy.screen_buffer());

        let image = Image::create_from_pixels(TV_WIDTH, TV_HEIGHT, &pixels).expect("image");

        texture.update_from_image(&image, 0, 0);

        window.draw(&Sprite::with_texture(&texture));

        window.display();
    }
}

fn convert_to_rgba(data: Vec<u8>) -> Vec<u8> {
    let mut result = vec![0; data.len() * 4];

    for (i, &color) in data.iter().enumerate() {
        let i = i * 4;
        let reduced = (color as f32 * 0.8) as u8;
        result[i] = reduced;
        result[i + 1] = color;
        result[i + 2] = reduced;
        result[i + 3] = 0xff;
    }

    result
}
