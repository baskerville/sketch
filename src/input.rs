extern crate libc;

use std::sync::mpsc::{Sender, Receiver};
use std::collections::HashMap;
use std::slice;
use std::mem;
use std::io::Read;
use std::os::unix::io::AsRawFd;
use std::fs::File;
use std::thread;
use std::sync::mpsc;
use geom::Point;

// Event types
pub const EV_SYN: u16 = 0;
pub const EV_KEY: u16 = 1;
pub const EV_ABS: u16 = 3;

// Event codes
pub const SYN_MT_REPORT: u16 = 2;
pub const ABS_MT_TRACKING_ID: u16 = 57;
pub const ABS_MT_TOUCH_MAJOR: u16 = 48;
pub const ABS_MT_POSITION_X: u16 = 53;
pub const ABS_MT_POSITION_Y: u16 = 54;
pub const KEY_POWER: u16 = 116;

#[repr(C)]
pub struct InputEvent {
    pub time: libc::timeval,
    pub kind: u16, // type
    pub code: u16,
    pub value: i32,
}

#[derive(Debug)]
pub enum FingerStatus {
    Down,
    Motion,
    Up,
}

#[derive(Debug)]
pub enum ButtonStatus {
    Pressed,
    Released,
}

#[derive(Debug)]
pub enum ButtonCode {
    Power,
}

#[derive(Debug)]
pub enum DeviceEvent {
    Finger {
        time: f64,
        id: i32,
        status: FingerStatus,
        position: Point,
    },
    Button {
        time: f64,
        code: ButtonCode,
        status: ButtonStatus,
    },
}

pub fn seconds(time: libc::timeval) -> f64 {
    time.tv_sec as f64 + time.tv_usec as f64 / 1e6
}

pub struct Input {
    pub events: Receiver<DeviceEvent>,
    pub dims: (u32, u32),
}

impl Input {
    pub fn new(paths: Vec<String>, dims: (u32, u32)) -> Input {
        let events = device_events(raw_events(paths), dims);
        Input {
            events: events,
            dims: dims,
        }
    }
}

pub fn raw_events(paths: Vec<String>) -> Receiver<InputEvent> {
    let (tx, rx) = mpsc::channel();
    thread::spawn(move || parse_raw_events(paths, tx));
    rx
}

pub fn parse_raw_events(paths: Vec<String>, tx: Sender<InputEvent>) {
    let mut files = Vec::new();
    let mut pfds = Vec::new();
    for path in paths.iter() {
        let file = File::open(path).unwrap();
        let fd = file.as_raw_fd();
        files.push(file);
        pfds.push(libc::pollfd {
            fd: fd,
            events: libc::POLLIN,
            revents: 0,
        });
    }
    loop {
        let ret = unsafe { libc::poll(pfds.as_mut_ptr(), pfds.len() as libc::nfds_t, -1) };
        if ret < 0 {
            break;
        }
        for (pfd, mut file) in pfds.iter().zip(&files) {
            if pfd.revents & libc::POLLIN != 0 {
                let mut input_event: InputEvent = unsafe { mem::uninitialized() };
                unsafe {
                    let event_slice = slice::from_raw_parts_mut(&mut input_event as *mut InputEvent as *mut u8,
                                                                mem::size_of::<InputEvent>());
                    if file.read_exact(event_slice).is_err() {
                        break;
                    }
                }
                tx.send(input_event).unwrap();
            }
        }
    }
}

pub fn device_events(rx: Receiver<InputEvent>, dims: (u32, u32)) -> Receiver<DeviceEvent> {
    let (ty, ry) = mpsc::channel();
    thread::spawn(move || parse_device_events(rx, ty, dims));
    ry
}

pub fn parse_device_events(rx: Receiver<InputEvent>, ty: Sender<DeviceEvent>, dims: (u32, u32)) {
    let mut id = -1;
    let mut position = Point::default();
    let mut pressure = 0;
    let mut fingers: HashMap<i32, Point> = HashMap::new();
    while let Ok(evt) = rx.recv() {
        if evt.kind == EV_ABS {
            if evt.code == ABS_MT_TRACKING_ID {
                id = evt.value;
            } else if evt.code == ABS_MT_TOUCH_MAJOR {
                pressure = evt.value;
            } else if evt.code == ABS_MT_POSITION_X {
                position.y = evt.value;
            } else if evt.code == ABS_MT_POSITION_Y {
                position.x = dims.0 as i32 - 1 - evt.value;
            }
        } else if evt.kind == EV_SYN {
            if evt.code == SYN_MT_REPORT {
                if let Some(&p) = fingers.get(&id) {
                    if pressure > 0 {
                        if p != position {
                            ty.send(DeviceEvent::Finger {
                                time: seconds(evt.time),
                                id: id,
                                status: FingerStatus::Motion,
                                position: position,
                            }).unwrap();
                        }
                    } else {
                        ty.send(DeviceEvent::Finger {
                            time: seconds(evt.time),
                            id: id,
                            status: FingerStatus::Up,
                            position: position,
                        }).unwrap();
                        fingers.remove(&id);
                    }
                } else {
                    ty.send(DeviceEvent::Finger {
                        time: seconds(evt.time),
                        id: id,
                        status: FingerStatus::Down,
                        position: position,
                    }).unwrap();
                    fingers.insert(id, position);
                }
            }
        } else if evt.kind == EV_KEY {
            if evt.code == KEY_POWER {
                ty.send(DeviceEvent::Button {
                    time: seconds(evt.time),
                    code: ButtonCode::Power,
                    status: if evt.value == 1 { ButtonStatus::Pressed } else
                                              { ButtonStatus::Released },
                }).unwrap();
            }
        }
    }
}

#[derive(Debug)]
enum TouchStatus {
    Begin,
    End
}

#[derive(Debug)]
enum Dir {
    North,
    East,
    South,
    West,
}

#[derive(Debug)]
enum GestureEvent {
    Touch {
        status: TouchStatus,
        position: Point,
    },
    Tap {
        finger_count: usize,
        position: Point,
    },
    Swipe {
        finger_count: usize,
        dir: Dir,
    },
    Pinch,
    Spread,
    Rotate,
}
