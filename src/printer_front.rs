use std::cell::RefCell;
use std::path::PathBuf;
use std::rc::Rc;

use mizu_core::Printer;

use native_dialog::FileDialog;
use sfml::{
    graphics::{Color, Image, RenderTarget, RenderWindow, Sprite, Texture},
    window::{Event, Key, Style},
};

use crate::{convert_to_rgba, update_window_view, TV_HEIGHT, TV_WIDTH};

pub struct MizuPrinter {
    printer: Rc<RefCell<Printer>>,
    window: RenderWindow,
    window_scroll: u32,
    pixels_buffer: [u8; TV_WIDTH as usize * TV_HEIGHT as usize * 4],
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
            pixels_buffer: [0xFF; TV_HEIGHT as usize * TV_WIDTH as usize * 4],
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

        let mut texture = Texture::new(TV_WIDTH, TV_HEIGHT).expect("texture");

        // the number of pixels to skip on scroll, each row is 160 pixels, each
        // pixel is 3 bytes
        let scroll_skip_pixels = self.window_scroll as usize * 160 * 3;
        let printer_image_buffer_view = &printer_image_buffer[scroll_skip_pixels..];

        // printer will output RGB but SFML need RGBA, this will map the
        // `printer_image_buffer_view` to `pixels_buffer` if the printer buffer
        // is smaller than pixels buffer or if its empty, it will only consume
        // the amount needed (because we use `zip` inside `convert_to_rgba`)
        // and zip will group two values until one of the two lists finish
        convert_to_rgba(printer_image_buffer_view, &mut self.pixels_buffer);

        let image =
            Image::create_from_pixels(TV_WIDTH, TV_HEIGHT, &self.pixels_buffer).expect("image");

        texture.update_from_image(&image, 0, 0);
        let sprite = Sprite::with_texture(&texture);

        self.window.clear(Color::BLACK);
        self.window.draw(&sprite);

        self.window.display();

        false
    }
}

impl MizuPrinter {
    /// fills the buffer with white color
    fn clear_pixels_buffer(&mut self) {
        for i in self.pixels_buffer.iter_mut() {
            *i = 0xFF;
        }
    }

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
                        self.clear_pixels_buffer();
                        self.window_scroll = 0;
                    }
                    Key::S => {
                        let file_dialog = FileDialog::new()
                            .set_filename("print.png")
                            .add_filter("PNG Image", &["png"]);

                        if let Ok(Some(filename)) = file_dialog.show_save_single_file() {
                            self.save_buffer_image_to_file(filename);
                        }
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

    fn save_buffer_image_to_file(&self, file_path: PathBuf) {
        let printer = self.printer.borrow();
        let printer_image_buffer = printer.get_image_buffer();
        let (width, height) = printer.get_image_size();

        if width * height != 0 {
            let mut result_image_buffer = vec![0xFF; width as usize * height as usize * 4];

            convert_to_rgba(printer_image_buffer, &mut result_image_buffer);

            let image =
                Image::create_from_pixels(width, height, &result_image_buffer).expect("image");

            let did_save = image.save_to_file(file_path.to_str().expect("PathBuf to_str"));

            if !did_save {
                println!("[ERROR] was not able to save image due to unknown reason");
            }
        } else {
            println!("[ERROR] cannot save an empty image");
        }
    }
}
