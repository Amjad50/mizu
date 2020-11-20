use gb_emu_core::GameBoy;
use std::env::args;

use sfml::{
    graphics::{Color, FloatRect, Image, RenderTarget, RenderWindow, Sprite, Texture, View},
    system::{SfBox, Vector2f},
    window::{Event, Key, Style},
};

const TV_WIDTH: u32 = 160;
const TV_HEIGHT: u32 = 144;

const SCREEN_WIDTH: u32 = TV_WIDTH * 3;
const SCREEN_HEIGHT: u32 = TV_HEIGHT * 3;

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
                Event::Resized { width, height } => {
                    window.set_view(&get_view(width, height, TV_WIDTH as u32, TV_HEIGHT as u32));
                }
                _ => {}
            }
        }

        for _ in 0..100000 {
            gameboy.clock();
        }

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
