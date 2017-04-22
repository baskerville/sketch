extern crate libc;
extern crate chrono;
extern crate png;

#[macro_use]
mod geom;
mod device;
mod input;
mod framebuffer;
mod sketch;

use sketch::Sketch;

fn main() {
    let mut sketch = Sketch::new();
    sketch.run();
}
