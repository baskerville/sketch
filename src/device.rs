extern crate libc;

use input::TouchProto;

#[derive(Debug)]
pub enum Model {
    Touch,
    Glo,
    Mini,
    AuraHD,
    Aura,
    AuraH2O,
    GloHD,
    Touch2,
    AuraONE,
    AuraEdition2,
}

#[derive(Debug)]
pub struct Device {
    pub model: Model,
    pub proto: TouchProto,
    pub swap_xy: bool,
    pub dpi: u16,
}

impl Default for Device {
    fn default() -> Device {
        Device {
            model: Model::Touch,
            proto: TouchProto::Single,
            swap_xy: false,
            dpi: 167,
        }
    }
}

impl Device {
    pub fn current() -> Device {
        let product = unsafe {
            let io = libc::popen("/bin/kobo_config.sh 2> /dev/null\0".as_ptr() as *const libc::c_char,
                                 "r\0".as_ptr() as *const libc::c_char);
            if io.is_null() {
                "trilogy".to_owned()
            } else {
                let mut buf = [0u8; 16];
                let result = if !libc::fgets(buf.as_mut_ptr() as *mut libc::c_char,
                                             buf.len() as libc::c_int, io).is_null() {
                    let len = buf.iter().position(|&v| v == 0).unwrap_or(0);
                    String::from_utf8_lossy(&buf[..len]).trim_right().to_owned()
                } else {
                    "trilogy".to_owned()
                };
                libc::pclose(io);
                result
            }
        };
        match product.as_ref() {
            "kraken" => Device {
                model: Model::Glo,
                proto: TouchProto::Single,
                swap_xy: true,
                dpi: 212,
            },
            "pixie" => Device {
                model: Model::Mini,
                proto: TouchProto::Single,
                swap_xy: true,
                dpi: 200,
            },
            "dragon" => Device {
                model: Model::AuraHD,
                proto: TouchProto::Single,
                swap_xy: true,
                dpi: 265,
            },
            "phoenix" => Device {
                model: Model::Aura,
                proto: TouchProto::Multi,
                swap_xy: true,
                dpi: 212,
            },
            "dahlia" => Device {
                model: Model::AuraH2O,
                proto: TouchProto::Multi,
                swap_xy: true,
                dpi: 265,
            },
            "alyssum" => Device {
                model: Model::GloHD,
                proto: TouchProto::Multi,
                swap_xy: true,
                dpi: 300,
            },
            "pika" => Device {
                model: Model::Touch2,
                proto: TouchProto::Multi,
                swap_xy: true,
                dpi: 167,
            },
            "daylight" => Device {
                model: Model::AuraONE,
                proto: TouchProto::Multi,
                swap_xy: true,
                dpi: 300,
            },
            "star" => Device {
                model: Model::AuraEdition2,
                proto: TouchProto::Multi,
                swap_xy: true,
                dpi: 212,
            },
            _ => Device::default(),
        }
    }
}
