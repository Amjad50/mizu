use std::env::args;
use std::time::{Duration, Instant};

use druid::keyboard_types::Key;
use druid::piet::{ImageFormat, InterpolationMode};
use druid::widget::prelude::*;
use druid::widget::{Align, Label};
use druid::{
    commands, AppLauncher, Data, FileDialogOptions, FileSpec, LocalizedString, MenuDesc, MenuItem,
    Point, Rect, Selector, TimerToken, WindowDesc,
};
use std::cell::RefCell;
use std::rc::Rc;

use gb_emu_core::{GameBoy, JoypadButton};

const TV_WIDTH: u32 = 160;
const TV_HEIGHT: u32 = 144;

const RESET: Selector = Selector::new("gb-emu_cmd_reset");
const ERROR: Selector<String> = Selector::new("gb-emu_cmd_error");

#[derive(Clone)]
struct GameBoyData {
    gameboy: Rc<RefCell<GameBoy>>,
    last_updated: Instant,
    screen_buffer: [u8; TV_HEIGHT as usize * TV_WIDTH as usize * 3],
}

impl Data for GameBoyData {
    fn same(&self, _other: &Self) -> bool {
        // screen should always be refereshed
        false
    }
}

impl GameBoyData {
    fn new(gameboy: GameBoy) -> Self {
        Self {
            gameboy: Rc::new(RefCell::new(gameboy)),
            last_updated: Instant::now(),
            screen_buffer: [0; TV_HEIGHT as usize * TV_WIDTH as usize * 3],
        }
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

impl Default for GameBoyWidget {
    fn default() -> Self {
        Self {
            timer_id: TimerToken::INVALID,
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
                    self.timer_id = ctx.request_timer(Duration::from_micros(1000_000 / 60));

                    ctx.window().set_title(&format!(
                        "GB-emu - FPS: {}",
                        (1. / data.last_updated.elapsed().as_secs_f64()).round()
                    ));
                    data.last_updated = Instant::now();

                    let mut buffer = None;
                    // If the emulation is running less than 60FPS, then two
                    //  or more timers might occure while gameboy is still
                    //  executing, so we can't allow that, if it happened,
                    //  we just ignore it this time and wait for the next timer
                    if let Ok(mut gameboy) = data.gameboy.try_borrow_mut() {
                        gameboy.clock_for_frame();
                        gameboy.audio_buffer();

                        buffer = Some(gameboy.screen_buffer().to_owned());
                    }

                    if let Some(buffer) = buffer {
                        data.screen_buffer.copy_from_slice(&buffer);
                    }

                    ctx.request_paint();
                }
            }
            Event::KeyDown(key_event) => match &key_event.key {
                Key::Character(key) if !key_event.repeat => {
                    if let Some(key) = Self::convert_key_into_joypad(&key) {
                        data.gameboy.borrow_mut().press_joypad(key);
                    }
                }
                _ => {}
            },
            Event::KeyUp(key_event) => match &key_event.key {
                Key::Character(key) if !key_event.repeat => {
                    if let Some(key) = Self::convert_key_into_joypad(&key) {
                        data.gameboy.borrow_mut().release_joypad(key);
                    }
                }
                _ => {}
            },
            Event::Command(command) => {
                if command.is(commands::OPEN_FILE) {
                    if let Some(file) = command.get(commands::OPEN_FILE) {
                        match GameBoy::new(file.path(), None) {
                            Ok(gameboy) => {
                                data.gameboy.replace(gameboy);
                            }
                            Err(err) => {
                                let error = format!("{}", err);
                                ctx.submit_command(ERROR.with(error));
                            }
                        }
                    }
                } else if command.is(RESET) {
                    data.gameboy.borrow_mut().reset();
                } else if command.is(ERROR) {
                    if let Some(error) = command.get(ERROR) {
                        let error = error.to_string();

                        // FIXME: control window size to make it smaller for alerts
                        let window = WindowDesc::new(|| error_builder(error))
                            .title("Error")
                            .with_min_size((0., 0.));

                        ctx.new_window(window);
                    }
                }
            }
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

        ctx.draw_image(&image, rect, InterpolationMode::NearestNeighbor)
    }
}

fn error_builder(error: String) -> impl Widget<GameBoyData> {
    Align::centered(Label::new(error))
}

fn ui_builder() -> impl Widget<GameBoyData> {
    Align::centered(GameBoyWidget::default())
}

pub fn main() {
    let args = args().collect::<Vec<String>>();

    if args.len() < 2 {
        eprintln!("USAGE: {} <rom-file> <boot-rom-file>", args[0]);
        return;
    }

    let gameboy = GameBoy::new(&args[1], args.get(2)).unwrap();

    let open_file_cmd = commands::SHOW_OPEN_PANEL.with(
        FileDialogOptions::new().allowed_types(vec![FileSpec::new("Gameboy Rom", &["gb", "gbc"])]),
    );

    let window = WindowDesc::new(ui_builder)
        .window_size(Size {
            width: TV_WIDTH as f64 * 5.,
            height: TV_HEIGHT as f64 * 5.,
        })
        .resizable(true)
        .menu(
            MenuDesc::empty()
                .append(
                    MenuDesc::new(
                        LocalizedString::new("gb-emu_menu_file").with_placeholder("File"),
                    )
                    .append(MenuItem::new(
                        LocalizedString::new("gb-emu_menuitem_file_open").with_placeholder("Open"),
                        open_file_cmd,
                    )),
                )
                .append(
                    MenuDesc::new(
                        LocalizedString::new("gb-emu_menu_game").with_placeholder("Game"),
                    )
                    .append(MenuItem::new(
                        LocalizedString::new("gb-emu_menuitem_game_reset")
                            .with_placeholder("Reset"),
                        RESET,
                    )),
                ),
        )
        .title("GB-emu");

    AppLauncher::with_window(window)
        .use_simple_logger()
        .launch(GameBoyData::new(gameboy))
        .expect("launch failed");
}
