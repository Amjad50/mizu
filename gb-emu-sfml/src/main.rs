use gb_emu_core::GameBoy;
use std::env::args;

use sfml::{
    graphics::{Color, Image, RenderTarget, RenderWindow, Sprite, Texture, View},
    system::Vector2f,
    window::{Event, Key, Style},
};

const TV_WIDTH: u32 = 160;
const TV_HEIGHT: u32 = 144;

const SCREEN_WIDTH: u32 = TV_WIDTH * 3;
const SCREEN_HEIGHT: u32 = TV_HEIGHT * 3;

fn main() {
    let args = args().collect::<Vec<String>>();

    if args.len() < 2 {
        eprintln!("USAGE: {} <rom-file>", args[0]);
        return;
    }

    let mut gameboy = GameBoy::new(&args[1]).unwrap();

    let mut window = RenderWindow::new(
        (SCREEN_WIDTH, SCREEN_HEIGHT),
        "NES test",
        Style::CLOSE,
        &Default::default(),
    );
    window.set_vertical_sync_enabled(true);

    // to scale the view into the window
    // this view is in the size of the GB TV screen
    // but we can scale the window and all the pixels will be scaled
    // accordingly
    let view = View::new(
        Vector2f::new((TV_WIDTH / 2) as f32, (TV_HEIGHT / 2) as f32),
        Vector2f::new((TV_WIDTH) as f32, (TV_HEIGHT) as f32),
    );
    window.set_view(&view);

    let mut texture = Texture::new(TV_WIDTH, TV_HEIGHT).expect("texture");

    'main: loop {
        while let Some(event) = window.poll_event() {
            match event {
                Event::Closed
                | Event::KeyPressed {
                    code: Key::Escape, ..
                } => break 'main,
                _ => {}
            }
        }

        for i in 0..1000 {
            gameboy.clock();
        }

        window.clear(Color::BLACK);

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
        result[i] = color;
        result[i + 1] = color;
        result[i + 2] = color;
        result[i + 3] = 0xff;
    }

    result
}
