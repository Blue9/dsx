//! Mapping from DualSense state to an XUSB (Xbox 360) report.
//!
//! Kept free of vigem-client types so it compiles and tests on any platform;
//! the button constants below are the wire values from the XUSB protocol,
//! identical to vigem-client's `XButtons`.

use crate::dualsense::DsState;

pub const XUSB_DPAD_UP: u16 = 0x0001;
pub const XUSB_DPAD_DOWN: u16 = 0x0002;
pub const XUSB_DPAD_LEFT: u16 = 0x0004;
pub const XUSB_DPAD_RIGHT: u16 = 0x0008;
pub const XUSB_START: u16 = 0x0010;
pub const XUSB_BACK: u16 = 0x0020;
pub const XUSB_LTHUMB: u16 = 0x0040;
pub const XUSB_RTHUMB: u16 = 0x0080;
pub const XUSB_LB: u16 = 0x0100;
pub const XUSB_RB: u16 = 0x0200;
pub const XUSB_GUIDE: u16 = 0x0400;
pub const XUSB_A: u16 = 0x1000;
pub const XUSB_B: u16 = 0x2000;
pub const XUSB_X: u16 = 0x4000;
pub const XUSB_Y: u16 = 0x8000;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct XusbReport {
    pub buttons: u16,
    pub left_trigger: u8,
    pub right_trigger: u8,
    pub thumb_lx: i16,
    pub thumb_ly: i16,
    pub thumb_rx: i16,
    pub thumb_ry: i16,
}

/// Widen a raw stick byte (0..=255, 128 centered) to the XUSB i16 range.
fn axis(v: u8) -> i16 {
    ((v as i32 - 128) * 32767 / 127).clamp(-32768, 32767) as i16
}

/// Same, but flipped: DualSense Y grows downward, XUSB Y grows upward.
fn axis_inverted(v: u8) -> i16 {
    ((128 - v as i32) * 32767 / 127).clamp(-32768, 32767) as i16
}

pub fn map(s: &DsState) -> XusbReport {
    let mut buttons = 0u16;
    let mut set = |on: bool, bit: u16| {
        if on {
            buttons |= bit;
        }
    };
    set(s.dpad_up, XUSB_DPAD_UP);
    set(s.dpad_down, XUSB_DPAD_DOWN);
    set(s.dpad_left, XUSB_DPAD_LEFT);
    set(s.dpad_right, XUSB_DPAD_RIGHT);
    set(s.cross, XUSB_A);
    set(s.circle, XUSB_B);
    set(s.square, XUSB_X);
    set(s.triangle, XUSB_Y);
    set(s.l1, XUSB_LB);
    set(s.r1, XUSB_RB);
    set(s.l3, XUSB_LTHUMB);
    set(s.r3, XUSB_RTHUMB);
    set(s.create, XUSB_BACK);
    set(s.options, XUSB_START);
    set(s.ps, XUSB_GUIDE);

    XusbReport {
        buttons,
        left_trigger: s.l2,
        right_trigger: s.r2,
        thumb_lx: axis(s.lx),
        thumb_ly: axis_inverted(s.ly),
        thumb_rx: axis(s.rx),
        thumb_ry: axis_inverted(s.ry),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn neutral() -> DsState {
        DsState { lx: 128, ly: 128, rx: 128, ry: 128, ..DsState::default() }
    }

    #[test]
    fn neutral_maps_to_zero() {
        assert_eq!(map(&neutral()), XusbReport::default());
    }

    #[test]
    fn axis_extremes() {
        let s = DsState { lx: 0, ly: 0, rx: 255, ry: 255, ..neutral() };
        let r = map(&s);
        // Stick pushed left/up gives min X, max Y (XUSB up is positive).
        assert_eq!(r.thumb_lx, -32768);
        assert_eq!(r.thumb_ly, 32767);
        assert_eq!(r.thumb_rx, 32767);
        assert_eq!(r.thumb_ry, -32767);
    }

    #[test]
    fn face_buttons() {
        let s = DsState { cross: true, circle: true, square: true, triangle: true, ..neutral() };
        assert_eq!(map(&s).buttons, XUSB_A | XUSB_B | XUSB_X | XUSB_Y);
    }

    #[test]
    fn system_buttons() {
        let s = DsState { create: true, options: true, ps: true, ..neutral() };
        assert_eq!(map(&s).buttons, XUSB_BACK | XUSB_START | XUSB_GUIDE);
    }

    #[test]
    fn triggers_pass_through() {
        let s = DsState { l2: 100, r2: 255, ..neutral() };
        let r = map(&s);
        assert_eq!((r.left_trigger, r.right_trigger), (100, 255));
    }
}
