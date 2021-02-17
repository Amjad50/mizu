use std::cell::RefCell;
use std::rc::Rc;

use mizu_core::Printer;

use sfml::{
    graphics::{Color, Image, RenderTarget, RenderWindow, Sprite, Texture, Transformable},
    window::{Event, Key, Style},
};

use crate::{convert_to_rgba, update_window_view, TV_HEIGHT, TV_WIDTH};

pub struct MizuPrinter {
    printer: Rc<RefCell<Printer>>,
    window: RenderWindow,
    window_scroll: u32,
}

impl Default for MizuPrinter {
    fn default() -> Self {
        let printer = Rc::new(RefCell::new(Printer::default()));

        let mut window = RenderWindow::new(
            (TV_WIDTH * 5, TV_HEIGHT * 5),
            "mizu printer",
            Style::CLOSE | Style::RESIZE,
            &Default::default(),
        );
        let size = window.size();
        update_window_view(&mut window, size.x, size.y);

        // do not block on update (infinite framerate)
        window.set_framerate_limit(0);

        Self {
            printer,
            window,
            window_scroll: 0,
        }
    }
}

impl MizuPrinter {
    pub fn get_printer(&self) -> Rc<RefCell<Printer>> {
        self.printer.clone()
    }

    pub fn close(&mut self) {
        self.window.close();
        self.window_scroll = 0;
    }

    /// return `ture` if the printer should be disconnected (closed)
    pub fn update_printer_window(&mut self) -> bool {
        if self.handle_printer_key_inputs() {
            return true;
        }

        let printer = self.printer.borrow();

        let printer_image_buffer = printer.get_image_buffer();
        let (width, height) = printer.get_image_size();

        if width * height != 0 {
            let mut texture = Texture::new(width, height).expect("texture");
            let mut window_image_buffer = vec![0; width as usize * height as usize * 4];

            // printer will output RGB but SFML need RGBA
            convert_to_rgba(printer_image_buffer, &mut window_image_buffer);

            let image =
                Image::create_from_pixels(width, height, &window_image_buffer).expect("image");

            texture.update_from_image(&image, 0, 0);
            let mut sprite = Sprite::with_texture(&texture);
            sprite.set_position((0., -(self.window_scroll as f32)));

            self.window.draw(&sprite);
        } else {
            self.window.clear(Color::BLACK);
        }

        self.window.display();

        false
    }
}

impl MizuPrinter {
    /// return `ture` if the printer should be disconnected (closed)
    fn handle_printer_key_inputs(&mut self) -> bool {
        let max_scroll = self.get_max_printer_window_scroll() as i32;

        while let Some(event) = self.window.poll_event() {
            match event {
                Event::Closed => {
                    return true;
                }
                Event::Resized { width, height } => {
                    update_window_view(&mut self.window, width, height);
                }
                Event::KeyPressed { code: key, .. } => match key {
                    Key::C => {
                        // clear image buffer
                        self.printer.borrow_mut().clear_image_buffer();
                        self.window_scroll = 0;
                    }
                    _ => {}
                },
                Event::MouseWheelScrolled { delta, .. } => {
                    // we use sub to invert
                    // speed of scroll
                    let delta = delta as i32 * 5;
                    let scroll = self.window_scroll as i32;
                    let scroll = scroll - delta;
                    // if its negative, keep it zero
                    let scroll = scroll.max(0);
                    // don't exceed max scroll
                    let scroll = scroll.min(max_scroll);

                    self.window_scroll = scroll as u32;
                }
                _ => {}
            }
        }

        false
    }

    fn get_max_printer_window_scroll(&self) -> u32 {
        let (_w, h) = self.printer.borrow().get_image_size();

        // it will be zero if `TV_HEIGHT` is larger than `h`
        h.saturating_sub(TV_HEIGHT)
    }
}
