use bitflags::bitflags;

bitflags! {
    #[derive(Default)]
    pub struct Buttons1: u8 {
        const A = 0b00000001;
        const B = 0b00000010;
        const X = 0b00000100;
        const Y = 0b00001000;
        const L1 = 0b00010000;
        const L2 = 0b00100000;
        const R1 = 0b01000000;
        const R2 = 0b10000000;
    }
}
bitflags! {
    #[derive(Default)]
    pub struct Buttons2: u8 {
        const SELECT = 0b00000001;
        const START = 0b00000010;
        const LSTICK = 0b00000100;
        const RSTICK = 0b00001000;
        const UP = 0b00010000;
        const DOWN = 0b00100000;
        const LEFT = 0b01000000;
        const RIGHT = 0b10000000;
    }
}
bitflags! {
    #[derive(Default)]
    pub struct Buttons3: u8 {
        const HOME = 0b00000001;
    }
}

#[derive(Debug, Clone, Copy, Default)]
#[repr(C)]
#[repr(packed)]
pub struct ControllerMessage {
    pub id: u8,
    pub seq: u8,
    pub buttons1: Buttons1,
    pub buttons2: Buttons2,
    pub buttons3: Buttons3,
    // assumption: host order
    pub axis_lh: u16,
    pub axis_lv: u16,
    pub axis_rh: u16,
    pub axis_rv: u16,
}
impl ControllerMessage {
    pub fn from_slice_u8(bytes: &[u8]) -> Result<Self, ()> {
        if std::mem::size_of::<ControllerMessage>() != bytes.len() {
            Err(())
        } else {
            Ok(unsafe { std::mem::transmute_copy(&bytes) })
        }
    }
}
