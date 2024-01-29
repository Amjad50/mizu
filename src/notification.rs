use super::{get_new_view, TV_HEIGHT, TV_WIDTH};

use sfml::{
    graphics::{Drawable, Font, Rect, RenderTarget, Text, Transformable},
    SfBox,
};

const NOTIF_FONT_SIZE: u32 = 25;
const NOTIF_FONT_OUTLINE: f32 = 1.5;
const NOTIF_TEXT_SPACING: f32 = 2.;
const NOTIF_DURATION: f32 = 4.;
const NOTIF_DISAPPEAR_REMAIN_TIME: f32 = 0.5;

const FONT_TTF_FILE: &[u8] = include_bytes!("./resources/Inconsolata/Inconsolata-Regular.ttf");

pub struct Notifications {
    messages: Vec<(String, f32)>,
    font: SfBox<Font>,
    width: u32,
    height: u32,
}

impl Notifications {
    pub fn new() -> Self {
        Self {
            messages: Vec::new(),
            // Safety: the `font` data is `'static` so its valid until the `Font` is used
            font: unsafe { Font::from_memory(FONT_TTF_FILE).unwrap() },
            width: TV_WIDTH,
            height: TV_HEIGHT,
        }
    }

    pub fn update_size(&mut self, width: u32, height: u32) {
        self.width = width;
        self.height = height;
    }

    pub fn add_msg(&mut self, msg: &str) {
        self.messages.push((msg.to_owned(), NOTIF_DURATION));
    }

    pub fn update(&mut self, delta: f32) {
        self.messages.iter_mut().for_each(|(_, c)| *c -= delta);
        self.messages.retain(|(_, c)| c > &0.);
    }
}

impl Drawable for Notifications {
    fn draw<'a: 'shader, 'texture, 'shader, 'shader_texture>(
        &'a self,
        target: &mut dyn RenderTarget,
        states: &sfml::graphics::RenderStates<'texture, 'shader, 'shader_texture>,
    ) {
        if self.messages.is_empty() {
            return;
        }

        // get the view of the gameboy rendering
        let gb_view = get_new_view(self.width, self.height, TV_WIDTH, TV_HEIGHT);

        // save the current view to restore to it later
        let saved_view = target.view().to_owned();
        // create a new view for our text rendering
        let mut text_rendring_view = saved_view.to_owned();
        // use the gameboy viewport, but without any resizing
        let mut gb_viewport = gb_view.viewport();
        gb_viewport.width = 1.;
        gb_viewport.height = 1.;
        text_rendring_view.set_viewport(gb_viewport);

        // get the length of the gameboy rendering by using `target` as reference measures
        // this will give us the distance, the text need to be offsetted in order
        // to be at the bottom of the rendering display
        let Rect {
            height: down,
            width: view_width,
            ..
        } = target.viewport(&gb_view);
        let down_base = down as f32 - (NOTIF_FONT_SIZE as f32 * 0.5);
        let mut next_y_pos = down_base;

        // a helper to wrap the text into multiple lines if needed
        let wrap_text = |text: &mut Text| {
            let font = text.font().unwrap();
            let char_size = text.character_size();
            let s = text.string().to_rust_string();
            let mut current_offset = 0.;
            let mut word_begin = 0;
            let mut first_word = true;
            let mut chars = s.chars().collect::<Vec<_>>();

            let mut i = 0;
            loop {
                let c = chars[i];
                if c == '\n' {
                    current_offset = 0.;
                    first_word = true;
                } else {
                    if c == ' ' {
                        word_begin = i;
                        first_word = false;
                    }

                    let glyph = font.glyph(c as u32, char_size, false, NOTIF_FONT_OUTLINE);
                    current_offset += glyph.advance();

                    if !first_word && current_offset > view_width as f32 {
                        i = word_begin;
                        chars[i] = '\n';
                        current_offset = 0.;
                        first_word = true;
                    }
                }

                i += 1;
                if i == chars.len() {
                    break;
                }
            }

            text.set_string(&chars.iter().collect::<String>());
        };

        target.set_view(&text_rendring_view);

        for (msg, c) in self.messages.iter().rev() {
            if next_y_pos < 0. {
                break;
            }

            // TODO: find a way to store the Text in the struct so that we don't need
            //  to recreate it every frame (now we cannot because of lifetime issue)
            let mut text = Text::new(msg, &self.font, NOTIF_FONT_SIZE);
            text.set_outline_thickness(NOTIF_FONT_OUTLINE);

            wrap_text(&mut text);

            let bounds = text.local_bounds();

            next_y_pos -= bounds.height + NOTIF_TEXT_SPACING;

            text.set_position((NOTIF_FONT_SIZE as f32 / 2., next_y_pos));

            // if there are stuff above the border, then don't bother rendering them and get out

            if c < &NOTIF_DISAPPEAR_REMAIN_TIME {
                let ratio = 255. / NOTIF_DISAPPEAR_REMAIN_TIME;
                let alpha_decrease = (ratio * (NOTIF_DISAPPEAR_REMAIN_TIME - *c)).min(255.) as u8;

                let mut color = text.outline_color();
                color.a -= alpha_decrease;
                text.set_outline_color(color);
                let mut color = text.fill_color();
                color.a -= alpha_decrease;
                text.set_fill_color(color);
            }
            target.draw_text(&text, states);
        }

        target.set_view(&saved_view);
    }
}
