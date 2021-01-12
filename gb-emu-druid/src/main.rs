use std::env::args;
use std::time::Duration;

use druid::piet::ImageFormat;
use druid::widget::prelude::*;
use druid::widget::Flex;
use druid::{AppLauncher, Data, Point, Rect, TimerToken, WindowDesc};
use std::cell::RefCell;
use std::rc::Rc;

use gb_emu_core::{GameBoy, JoypadButton};

const TV_WIDTH: u32 = 160;
const TV_HEIGHT: u32 = 144;

#[derive(Clone)]
struct GameBoyData {
    gameboy: Rc<RefCell<GameBoy>>,
    screen_buffer: [u8; TV_HEIGHT as usize * TV_WIDTH as usize * 3],
}

impl Data for GameBoyData {
    fn same(&self, _other: &Self) -> bool {
        // screen should always be refereshed
        false
    }
}

struct GameBoyWidget {
    timer_id: TimerToken,
}

impl GameBoyWidget {
    fn convert_key_into_joypad(key: &str) -> Option<JoypadButton> {
        match key.chars().next().unwrap().to_ascii_uppercase() {
            'J' => Some(JoypadButton::B),
            'K' => Some(JoypadButton::A),
            'U' => Some(JoypadButton::Select),
            'I' => Some(JoypadButton::Start),
            'W' => Some(JoypadButton::Up),
            'S' => Some(JoypadButton::Down),
            'A' => Some(JoypadButton::Left),
            'D' => Some(JoypadButton::Right),
            _ => None,
        }
    }
}

impl Widget<GameBoyData> for GameBoyWidget {
    fn event(&mut self, ctx: &mut EventCtx, event: &Event, data: &mut GameBoyData, _env: &Env) {
        match event {
            Event::WindowConnected => {
                // focus is important to capture keyboard events
                ctx.request_focus();
                ctx.request_paint();
                self.timer_id = ctx.request_timer(Duration::from_micros(1000_000 / 60));
            }
            Event::Timer(id) => {
                if *id == self.timer_id {
                    let mut gameboy = data.gameboy.borrow_mut();
                    gameboy.clock_for_frame();
                    gameboy.audio_buffer();

                    let buffer = gameboy.screen_buffer().to_owned();

                    drop(gameboy);

                    data.screen_buffer.copy_from_slice(&buffer);

                    ctx.request_paint();
                    self.timer_id = ctx.request_timer(Duration::from_micros(1000_000 / 60));
                }
            }
            Event::KeyDown(key_event) => match &key_event.key {
                druid::keyboard_types::Key::Character(key) if !key_event.repeat => {
                    if let Some(key) = Self::convert_key_into_joypad(&key) {
                        data.gameboy.borrow_mut().press_joypad(key);
                    }
                }
                _ => {}
            },
            Event::KeyUp(key_event) => match &key_event.key {
                druid::keyboard_types::Key::Character(key) if !key_event.repeat => {
                    if let Some(key) = Self::convert_key_into_joypad(&key) {
                        data.gameboy.borrow_mut().release_joypad(key);
                    }
                }
                _ => {}
            },
            _ => {}
        }
    }

    fn lifecycle(
        &mut self,
        _ctx: &mut LifeCycleCtx,
        _event: &LifeCycle,
        _data: &GameBoyData,
        _env: &Env,
    ) {
    }

    fn update(
        &mut self,
        _ctx: &mut UpdateCtx,
        _old_data: &GameBoyData,
        _data: &GameBoyData,
        _env: &Env,
    ) {
    }

    fn layout(
        &mut self,
        _ctx: &mut LayoutCtx,
        bc: &BoxConstraints,
        _data: &GameBoyData,
        _env: &Env,
    ) -> Size {
        let max_size = bc.max();
        let width_perc = max_size.width / TV_WIDTH as f64;
        let heigt_perc = max_size.height / TV_HEIGHT as f64;
        let min_perc = heigt_perc.min(width_perc);
        Size {
            width: TV_WIDTH as f64 * min_perc,
            height: TV_HEIGHT as f64 * min_perc,
        }
    }

    fn paint(&mut self, ctx: &mut PaintCtx, data: &GameBoyData, _env: &Env) {
        let image = ctx
            .make_image(
                TV_WIDTH as usize,
                TV_HEIGHT as usize,
                &data.screen_buffer,
                ImageFormat::Rgb,
            )
            .unwrap();

        let rect = Rect::from_origin_size(Point::new(0., 0.), ctx.size());

        ctx.draw_image(
            &image,
            rect,
            druid::piet::InterpolationMode::NearestNeighbor,
        )
    }
}

fn ui_builder() -> impl Widget<GameBoyData> {
    Flex::column().with_flex_child(
        GameBoyWidget {
            timer_id: TimerToken::INVALID,
        },
        1.0,
    )
}

pub fn main() {
    let args = args().collect::<Vec<String>>();

    if args.len() < 2 {
        eprintln!("USAGE: {} <rom-file> <boot-rom-file>", args[0]);
        return;
    }

    let gameboy = GameBoy::new(&args[1], args.get(2)).unwrap();

    let window = WindowDesc::new(ui_builder)
        .window_size(Size {
            width: TV_WIDTH as f64 * 5.,
            height: TV_HEIGHT as f64 * 5.,
        })
        .resizable(true)
        .title("GB-emu");

    AppLauncher::with_window(window)
        .use_simple_logger()
        .launch(GameBoyData {
            gameboy: Rc::new(RefCell::new(gameboy)),
            screen_buffer: [0; TV_HEIGHT as usize * TV_WIDTH as usize * 3],
        })
        .expect("launch failed");
}
