use bitflags::bitflags;
use std::fmt;

bitflags! {
    #[derive(Default)]
    pub struct Buttons1: u8 {
        const A = 0b00000001;
        const B = 0b00000010;
        const X = 0b00000100;
        const Y = 0b00001000;
        const L1 = 0b00010000;
        const R1 = 0b00100000;
        const L2 = 0b01000000;
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

#[derive(Clone, Copy, Default)]
#[repr(C)]
#[repr(packed)]
pub struct ControllerMessage {
    pub id: u8,
    pub seq: u8,
    pub buttons1: Buttons1,
    pub buttons2: Buttons2,
    pub buttons3: Buttons3,
    pub axis_lh_hi: u8,
    pub axis_lh_lo: u8,
    pub axis_lv_hi: u8,
    pub axis_lv_lo: u8,
    pub axis_rh_hi: u8,
    pub axis_rh_lo: u8,
    pub axis_rv_hi: u8,
    pub axis_rv_lo: u8,
}
fn ntohs(hi: u8, lo: u8) -> u16 {
    u16::from_be_bytes([hi, lo])
}
impl fmt::Debug for ControllerMessage {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("ControllerMessage")
            .field("id", &self.id)
            .field("seq", &self.seq)
            .field("buttons1", &self.buttons1)
            .field("buttons2", &self.buttons2)
            .field("buttons3", &self.buttons3)
            .field("axis_lh", &ntohs(self.axis_lh_hi, self.axis_lh_lo))
            .field("axis_lv", &ntohs(self.axis_lv_hi, self.axis_lv_lo))
            .field("axis_rh", &ntohs(self.axis_rh_hi, self.axis_rh_lo))
            .field("axis_rv", &ntohs(self.axis_rv_hi, self.axis_rv_lo))
            .finish()
    }
}
impl ControllerMessage {
    pub fn from_slice_u8(bytes: &[u8]) -> Result<Self, ()> {
        if std::mem::size_of::<ControllerMessage>() != bytes.len() {
            Err(())
        } else {
            Ok(unsafe { std::ptr::read(bytes.as_ptr() as *const _) })
        }
    }
}
