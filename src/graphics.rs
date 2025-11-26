use core::sync::atomic::{AtomicBool, AtomicU8, Ordering};
use embassy_time::{Duration, Timer};
use embedded_graphics::pixelcolor::Gray4;
use embedded_graphics::prelude::*;

use crate::sprite::{
    draw_sheet_frame_flash, draw_sheet_frame_masked, draw_sheet_frame_masked_crt, frame_count,
};
use crate::volume_indicator::VolumeIndicator;
use crate::{adc, screen};

const FRAME_DELAY: u64 = 140;

static SPIDER_SHEET: &[u8] = include_bytes!("sprites/muffet.gray4");

const SPIDER_SHEET_W: u32 = 104;
const SPIDER_SHEET_H: u32 = 64;

static SPIDER_CLOSE_SHEET: &[u8] = include_bytes!("sprites/muffet_close.gray4");
const SPIDER_CLOSE_SHEET_W: u32 = 122;
const SPIDER_CLOSE_SHEET_H: u32 = 64;

static COBWEB_SHEET: &[u8] = include_bytes!("sprites/cobweb_rotating.gray4");
const COBWEB_W: u32 = 40;
const COBWEB_H: u32 = 40;
const COBWEB_COUNT: usize = 4;

const HALO_STEPS: u8 = 3;

#[derive(PartialEq, Eq)]
pub enum ScreenState {
    INTRO = 0,
    STANDBY = 1,
    ACTIVE = 2,
    OUTRO = 3,
    OFF = 4,
}

pub static SCREEN_STATE: AtomicU8 = AtomicU8::new(ScreenState::OFF as u8);
pub static ACTIVE_INPUT: AtomicBool = AtomicBool::new(false);

pub fn get_screen_state() -> ScreenState {
    match SCREEN_STATE.load(Ordering::Relaxed) {
        0 => ScreenState::INTRO,
        1 => ScreenState::STANDBY,
        2 => ScreenState::ACTIVE,
        3 => ScreenState::OUTRO,
        4 => ScreenState::OFF,
        _ => ScreenState::OFF,
    }
}

#[embassy_executor::task]
pub async fn prepare_frame_task() {
    let mut indicator: VolumeIndicator = VolumeIndicator::new(Point::new(170, 1));

    let mut background = Background::new(screen::SCREEN_WIDTH as i32, screen::SCREEN_HEIGHT as i32);

    let mut intro_screen = IntroScreen::new(
        SPIDER_CLOSE_SHEET,
        Point { x: 66, y: 64 },
        SPIDER_CLOSE_SHEET_W,
        SPIDER_CLOSE_SHEET_H,
    );
    let mut standby_screen = StandbyScreen::new(
        SPIDER_SHEET,
        Point { x: 0, y: 0 },
        SPIDER_SHEET_W,
        SPIDER_SHEET_H,
        151,
    );
    let mut active_channel_screen = ActiveChannelScreen::new(
        SPIDER_CLOSE_SHEET,
        Point { x: 15, y: 0 },
        SPIDER_CLOSE_SHEET_W,
        SPIDER_CLOSE_SHEET_H,
    );
    let mut outro_screen = OutroScreen::new(
        SPIDER_CLOSE_SHEET,
        Point { x: 66, y: 0 },
        SPIDER_CLOSE_SHEET_W,
        SPIDER_CLOSE_SHEET_H,
    );

    loop {
        let frame = screen::NEXT_FRAME.wait().await;
        frame.clear(Gray4::BLACK).unwrap();

        let mut state = get_screen_state();
        let active_channel = adc::get_active_channel();

        if active_channel.is_none() && state == ScreenState::ACTIVE {
            state = ScreenState::STANDBY;
            SCREEN_STATE.store(ScreenState::STANDBY as u8, Ordering::Relaxed);
        } else if active_channel.is_some() && state == ScreenState::STANDBY {
            state = ScreenState::ACTIVE;
            SCREEN_STATE.store(ScreenState::ACTIVE as u8, Ordering::Relaxed);
        }

        match state {
            ScreenState::INTRO => {
                background.draw(frame);
                intro_screen.draw(frame);

                if intro_screen.firework_time {
                    background.start_intro_halo(Point::new(108, 20));
                }
            }
            ScreenState::STANDBY => {
                background.draw(frame);
                standby_screen.draw(frame);
            }
            ScreenState::ACTIVE => {
                let idx = active_channel.unwrap_or(0);

                let adc = adc::read_adc_value(idx) as u16;

                active_channel_screen.draw(frame);
                indicator.draw(frame, adc, adc::ADC_CHANNELS[idx].target);
            }
            ScreenState::OUTRO => {
                outro_screen.draw(frame);
            }
            ScreenState::OFF => {}
        }

        screen::READY_FRAME.signal(frame);

        Timer::after(Duration::from_millis(FRAME_DELAY)).await;
    }
}

#[derive(Copy, Clone)]
struct Cobweb {
    pos: Point,
    vel: Point,
    frame: usize,
    is_respawned: bool,
}

impl Cobweb {
    fn new(pos: Point, vel: Point, frame: usize) -> Self {
        Self {
            pos,
            vel,
            frame,
            is_respawned: false,
        }
    }
}
enum BackgroundMode {
    Inactive,
    IntroHalo { step: u8 },
    Normal,
}

pub struct Background {
    sprite: &'static [u8],
    sprite_w: u32,
    sprite_h: u32,
    frame_total: usize,
    mode: BackgroundMode,

    screen_width: i32,
    screen_height: i32,

    cobwebs: [Cobweb; COBWEB_COUNT],

    rng_state: u32,

    frame_skip: u8,
    frame_counter: u8,
}

impl Background {
    pub fn new(screen_width: i32, screen_height: i32) -> Self {
        let sprite_w = COBWEB_W;
        let sprite_h = COBWEB_H;
        let frame_total = frame_count(COBWEB_SHEET, sprite_w, sprite_h);

        let lane_width = screen_width / COBWEB_COUNT as i32;

        let mut cobwebs = [Cobweb::new(Point::new(0, 0), Point::new(0, 1), 0); COBWEB_COUNT];
        for (i, web) in cobwebs.iter_mut().enumerate() {
            let lane_x = i as i32 * lane_width;

            let x = lane_x + lane_width / 2 - (sprite_w as i32 / 2);
            let y = -((sprite_h as i32) * i as i32);

            *web = Cobweb::new(Point::new(x, y), Point::new(0, 1), i % 4);
        }

        Self {
            sprite: COBWEB_SHEET,
            sprite_w,
            sprite_h,
            frame_total,
            screen_width,
            screen_height,
            cobwebs,
            rng_state: 0x1234_5678,
            frame_skip: 4,
            frame_counter: 0,
            mode: BackgroundMode::Inactive,
        }
    }

    pub fn draw<D>(&mut self, display: &mut D)
    where
        D: DrawTarget<Color = Gray4>,
    {
        match self.mode {
            BackgroundMode::Inactive => {}

            BackgroundMode::IntroHalo { ref mut step } => {
                for (_i, web) in self.cobwebs.iter_mut().enumerate() {
                    let _ = draw_sheet_frame_flash(
                        display,
                        self.sprite,
                        self.sprite_w,
                        self.sprite_h,
                        web.frame,
                        web.pos,
                        *step,
                        HALO_STEPS,
                    );
                }

                *step = step.saturating_add(1);

                if get_screen_state() != ScreenState::INTRO {
                    for i in 0..COBWEB_COUNT {
                        let r_speed = self.next_rand();
                        let r_drift = self.next_rand();

                        let speed_y = 2 + (r_speed % 4) as i32;

                        let drift_table = [-2, 0, 2];
                        let dx = drift_table[(r_drift % 3) as usize];

                        let web = &mut self.cobwebs[i];
                        web.pos = web.pos;
                        web.vel = Point::new(dx, speed_y);
                    }
                    self.mode = BackgroundMode::Normal;
                } else if *step > HALO_STEPS {
                    for (_i, web) in self.cobwebs.iter_mut().enumerate() {
                        web.frame = (web.frame + 1) % self.frame_total;
                    }
                }
            }

            BackgroundMode::Normal => {
                // your existing falling logic, with frame_skip etc
                self.frame_counter = (self.frame_counter + 1) % self.frame_skip;

                let mut to_respawn: [bool; COBWEB_COUNT] = [false; COBWEB_COUNT];

                for i in 0..COBWEB_COUNT {
                    let web = &mut self.cobwebs[i];

                    let _ = draw_sheet_frame_masked(
                        display,
                        self.sprite,
                        self.sprite_w,
                        self.sprite_h,
                        web.frame,
                        web.pos,
                    );

                    if self.frame_counter > 0 && web.is_respawned {
                        continue;
                    }

                    web.frame = (web.frame + 1) % self.frame_total;
                    web.pos += web.vel;

                    if web.pos.y > self.screen_height + self.sprite_h as i32
                        || web.pos.x > self.screen_width + self.sprite_w as i32
                        || web.pos.x < -(self.sprite_w as i32)
                    {
                        to_respawn[i] = true;
                    }
                }

                for i in 0..COBWEB_COUNT {
                    if to_respawn[i] {
                        self.respawn_at_top(i);
                    }
                }
            }
        }
    }

    fn respawn_at_top(&mut self, index: usize) {
        let lane_width = self.screen_width / COBWEB_COUNT as i32;

        let r_lane = self.next_rand();
        let r_y = self.next_rand();
        let r_speed = self.next_rand();
        let r_drift = self.next_rand();
        let r_frame = self.next_rand();

        let lane_idx = (r_lane as usize) % COBWEB_COUNT;
        let lane_x = lane_idx as i32 * lane_width;
        let x = lane_x + lane_width / 2 - (self.sprite_w as i32 / 2);

        let max_offset = self.screen_height + self.sprite_h as i32;
        let offset = (r_y % (max_offset as u32)) as i32;
        let y = -offset;

        let speed_y = 2 + (r_speed % 4) as i32;

        let drift_table = [-2, 0, 2];
        let dx = drift_table[(r_drift % 3) as usize];

        let web = &mut self.cobwebs[index];
        web.pos = Point::new(x, y);
        web.vel = Point::new(dx, speed_y);
        web.is_respawned = true;

        web.frame = (r_frame as usize) % self.frame_total;
    }

    fn next_rand(&mut self) -> u32 {
        self.rng_state = self
            .rng_state
            .wrapping_mul(1_664_525)
            .wrapping_add(1_013_904_223);
        self.rng_state
    }

    pub fn start_intro_halo(&mut self, origin: Point) {
        // Four positions around Muffet; tweak offsets to taste
        let offsets = [
            Point::new(-90, -15), // left-up
            Point::new(90, -15),  // right-up
            Point::new(-60, 15),  // left-down
            Point::new(60, 15),   // right-down
        ];

        for (i, web) in self.cobwebs.iter_mut().enumerate() {
            let idx = i % offsets.len();
            web.pos = origin + offsets[idx];
            web.vel = Point::new(0, 0); // no movement during halo
            web.frame = i % self.frame_total;
            web.is_respawned = false;
        }

        self.mode = BackgroundMode::IntroHalo { step: 0 };
    }
}

struct IntroScreen {
    sprite: &'static [u8],
    start_coords: Point,
    coords: Point,
    sprite_w: u32,
    sprite_h: u32,
    frame: usize,
    frame_total: usize,
    intro_frame: usize,
    intro_frame_total: usize,
    firework_frame: usize,
    firework_time: bool,
}

impl IntroScreen {
    pub fn new(sprite: &'static [u8], start_coords: Point, sprite_w: u32, sprite_h: u32) -> Self {
        Self {
            sprite,
            start_coords,
            coords: start_coords,
            sprite_w,
            sprite_h,
            frame: 0,
            frame_total: frame_count(sprite, sprite_w, sprite_h),
            intro_frame: 0,
            intro_frame_total: 26,
            firework_frame: 8,
            firework_time: false,
        }
    }

    pub fn draw<D>(&mut self, display: &mut D)
    where
        D: DrawTarget<Color = Gray4>,
    {
        let _ = draw_sheet_frame_masked(
            display,
            self.sprite,
            self.sprite_w,
            self.sprite_h,
            self.frame,
            self.coords,
        );

        self.frame = (self.frame + 1) % self.frame_total;
        self.intro_frame += 1;

        if self.coords.y > 0 {
            self.coords -= Point::new(0, 8);
        } else {
            self.coords = Point::new(self.start_coords.x, 0);
        }

        if self.intro_frame == self.firework_frame {
            self.firework_time = true;
        } else {
            self.firework_time = false;
        }

        if self.intro_frame >= self.intro_frame_total {
            SCREEN_STATE.store(ScreenState::STANDBY as u8, Ordering::Relaxed);
            self.coords = self.start_coords;
            self.intro_frame = 0;
            self.frame = 0;
        }
    }
}

struct StandbyScreen {
    sprite: &'static [u8],
    width: u32,
    coords: Point,
    sprite_w: u32,
    sprite_h: u32,
    frame: usize,
    frame_total: usize,
    direction: bool,
}

impl StandbyScreen {
    pub fn new(
        sprite: &'static [u8],
        coords: Point,
        sprite_w: u32,
        sprite_h: u32,
        width: u32,
    ) -> Self {
        Self {
            sprite,
            coords,
            width,
            sprite_w,
            sprite_h,
            frame: 0,
            frame_total: frame_count(sprite, sprite_w, sprite_h),
            direction: true,
        }
    }

    pub fn draw<D>(&mut self, display: &mut D)
    where
        D: DrawTarget<Color = Gray4>,
    {
        let _ = draw_sheet_frame_masked(
            display,
            self.sprite,
            self.sprite_w,
            self.sprite_h,
            self.frame,
            self.coords,
        );

        self.frame = (self.frame + 1) % self.frame_total;

        if self.direction {
            self.coords += Point::new(1, 0);
        } else {
            self.coords -= Point::new(1, 0);
        }
        if self.coords.x >= self.width as i32 || self.coords.x <= 0 {
            self.direction = !self.direction;
        }
    }
}

struct ActiveChannelScreen {
    sprite: &'static [u8],
    coords: Point,
    sprite_w: u32,
    sprite_h: u32,
    frame: usize,
    frame_total: usize,
    frame_global: u8,
}

impl ActiveChannelScreen {
    pub fn new(sprite: &'static [u8], coords: Point, sprite_w: u32, sprite_h: u32) -> Self {
        Self {
            sprite,
            coords,
            sprite_w,
            sprite_h,
            frame: 0,
            frame_total: frame_count(sprite, sprite_w, sprite_h),
            frame_global: 0,
        }
    }

    pub fn draw<D>(&mut self, display: &mut D)
    where
        D: DrawTarget<Color = Gray4>,
    {
        let _ = draw_sheet_frame_masked_crt(
            display,
            self.sprite,
            self.sprite_w,
            self.sprite_h,
            self.frame,
            self.coords,
            self.frame_global,
            ACTIVE_INPUT.load(Ordering::Relaxed),
        );
        self.frame_global = self.frame_global.wrapping_add(1);

        self.frame = (self.frame + 1) % self.frame_total;
    }
}

struct OutroScreen {
    sprite: &'static [u8],
    start_coords: Point,
    coords: Point,
    sprite_w: u32,
    sprite_h: u32,
    frame: usize,
    frame_total: usize,
    fade_step: u8,
    fade_steps: u8,
}

impl OutroScreen {
    pub fn new(sprite: &'static [u8], start_coords: Point, sprite_w: u32, sprite_h: u32) -> Self {
        Self {
            sprite,
            start_coords,
            coords: start_coords,
            sprite_w,
            sprite_h,
            frame: 0,
            frame_total: frame_count(sprite, sprite_w, sprite_h),
            fade_step: 0,
            fade_steps: 16,
        }
    }

    pub fn draw<D>(&mut self, display: &mut D)
    where
        D: DrawTarget<Color = Gray4>,
    {
        let _ = crate::sprite::draw_sheet_frame_fade_dither(
            display,
            self.sprite,
            self.sprite_w,
            self.sprite_h,
            self.frame,
            self.coords,
            self.fade_step,
            self.fade_steps,
        );

        self.frame = (self.frame + 1) % self.frame_total;

        if self.fade_step < self.fade_steps {
            self.fade_step += 1;
        } else {
            self.coords = self.start_coords;
            self.fade_step = 0;
            self.frame = 0;
            SCREEN_STATE.store(ScreenState::OFF as u8, Ordering::Relaxed);
        }
    }
}
