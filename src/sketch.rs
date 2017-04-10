extern crate chrono;
extern crate libc;
extern crate png;

use std::collections::HashMap;
use framebuffer::{Framebuffer, Mode};
use input::{Input, DeviceEvent, FingerStatus, ButtonStatus, ButtonCode};
use geom::{Point, Rectangle};

const UPDATE_INTERVAL: f64 = 1.0 / 60.0;
const SAVE_INTERVAL: f64 = 5.0;

pub struct Sketch {
    fb: Framebuffer,
    input: Input,
}

struct TouchState {
    pt: Point,
    rect: Rectangle,
}

impl TouchState {
    fn new(pt: Point, rect: Rectangle) -> TouchState {
        TouchState {
            pt: pt,
            rect: rect,
        }
    }
}

impl Sketch {
    pub fn new() -> Sketch {
        let fb = Framebuffer::new("/dev/fb0").unwrap();
        let input = Input::new(vec!["/dev/input/event0".to_owned(),
                                    "/dev/input/event1".to_owned()],
                               fb.dims());
        Sketch {
            fb: fb,
            input: input,
        }
    }
    pub fn run(&mut self) {
        let mut fingers: HashMap<i32, TouchState> = HashMap::new();
        let mut last_update_time = 0.0;
        let mut last_save_time = 0.0;
        self.clear();
        while let Ok(evt) = self.input.events.recv() {
            match evt {
                DeviceEvent::Finger { status: FingerStatus::Motion, id, position, time } => {
                    if let Some(ts) = fingers.get_mut(&id) {
                        ts.rect.merge(&position);
                        self.fb.draw_line_segment(&position, &ts.pt, 0x00);
                        if (time - last_update_time).abs() > UPDATE_INTERVAL {
                            self.fb.update(ts.rect, Mode::Fast).unwrap();
                            last_update_time = time;
                            ts.rect = Rectangle::from_point(&position);
                        }
                        ts.pt = position;
                    }
                },
                DeviceEvent::Finger { status: FingerStatus::Down, id, position, .. } => {
                    fingers.insert(id, TouchState::new(position, Rectangle::from_point(&position)));
                },
                DeviceEvent::Finger { status: FingerStatus::Up, id, .. } => {
                    fingers.remove(&id);
                },
                DeviceEvent::Button { status: ButtonStatus::Pressed, code: ButtonCode::Power, time } => {
                    if (time - last_save_time).abs() < SAVE_INTERVAL {
                        break;
                    }
                    last_save_time = time;
                    self.save();
                    self.clear();
                },
                _ => (),
            }
        }
    }
    pub fn clear(&mut self) {
        let (width, height) = self.fb.dims();
        for x in 0..width {
            for y in 0..height {
                self.fb.set_pixel(x, y, 0xff);
            }
        }
        self.fb.update(rect!(0, 0, width as i32, height as i32), Mode::Full).unwrap();
    }

    pub fn save(&mut self) {
        self.fb.save(format!("drawing-{}.png", chrono::Local::now().format("%Y%m%d_%H%M%S").to_string()));
    }
}
