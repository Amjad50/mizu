use std::env::args;
use std::ops::{Index, IndexMut};
use std::time::{Duration, Instant};

use druid::piet::ImageFormat;
use druid::widget::prelude::*;
use druid::widget::{Button, Controller, Flex, Image, Label, Slider};
use druid::{
    AppLauncher, Color, Data, ImageBuf, Lens, LocalizedString, MouseButton, Point, Rect,
    TimerToken, WidgetExt, WindowDesc,
};
use std::cell::RefCell;
use std::rc::Rc;
use std::sync::Arc;

use gb_emu_core::{GameBoy, JoypadButton};

const TV_WIDTH: u32 = 160;
const TV_HEIGHT: u32 = 144;

#[derive(Clone)]
struct GameBoyData {
    gameboy: Rc<RefCell<GameBoy>>,
    screen_buffer: [u8; TV_HEIGHT as usize * TV_WIDTH as usize * 3],
}

impl Data for GameBoyData {
    fn same(&self, other: &Self) -> bool {
        false
    }
}

struct GameBoyWidget {
    timer_id: TimerToken,
}

impl GameBoyWidget {}

impl Widget<GameBoyData> for GameBoyWidget {
    fn event(&mut self, ctx: &mut EventCtx, event: &Event, data: &mut GameBoyData, env: &Env) {
        match event {
            Event::WindowConnected => {
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
            Event::KeyDown(key_event) => {
                //let mut gameboy = data.gameboy.borrow_mut();
                println!("key down");

                match &key_event.key {
                    druid::keyboard_types::Key::Character(key) => {
                        println!("{}", key);
                    }
                    _ => {}
                }
            }
            Event::KeyUp(key_event) => {
                //let mut gameboy = data.gameboy.borrow_mut();
                println!("key down");

                match &key_event.key {
                    druid::keyboard_types::Key::Character(key) => {
                        println!("{}", key);
                    }
                    _ => {}
                }
            }
            _ => {}
        }
    }

    fn lifecycle(
        &mut self,
        ctx: &mut LifeCycleCtx,
        event: &LifeCycle,
        data: &GameBoyData,
        env: &Env,
    ) {
    }

    fn update(
        &mut self,
        ctx: &mut UpdateCtx,
        old_data: &GameBoyData,
        data: &GameBoyData,
        env: &Env,
    ) {
    }

    fn layout(
        &mut self,
        ctx: &mut LayoutCtx,
        bc: &BoxConstraints,
        data: &GameBoyData,
        env: &Env,
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

    fn paint(&mut self, ctx: &mut PaintCtx, data: &GameBoyData, env: &Env) {
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

struct GameBoyController {}

impl Controller<GameBoyData, GameBoyWidget> for GameBoyController {
    fn event(
        &mut self,
        child: &mut GameBoyWidget,
        ctx: &mut EventCtx,
        event: &Event,
        data: &mut GameBoyData,
        env: &Env,
    ) {
        child.event(ctx, event, data, env)
    }

    fn lifecycle(
        &mut self,
        child: &mut GameBoyWidget,
        ctx: &mut LifeCycleCtx,
        event: &LifeCycle,
        data: &GameBoyData,
        env: &Env,
    ) {
        child.lifecycle(ctx, event, data, env)
    }

    fn update(
        &mut self,
        child: &mut GameBoyWidget,
        ctx: &mut UpdateCtx,
        old_data: &GameBoyData,
        data: &GameBoyData,
        env: &Env,
    ) {
        child.update(ctx, old_data, data, env)
    }
}

fn ui_builder() -> impl Widget<GameBoyData> {
    Flex::column().with_flex_child(
        GameBoyWidget {
            timer_id: TimerToken::INVALID,
        }
        .controller(GameBoyController {}),
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
