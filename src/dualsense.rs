//! Parsing of DualSense HID input reports.
//!
//! The controller sends three report layouts:
//! - USB: report ID 0x01, 64 bytes, full layout starting at byte 1
//! - Bluetooth (default): report ID 0x01, 10 bytes, compact layout
//! - Bluetooth (extended): report ID 0x31, full layout starting at byte 2
//!
//! Sony's Vendor ID is 0x054C; the DualSense is PID 0x0CE6 and the
//! DualSense Edge is PID 0x0DF2.

pub const SONY_VID: u16 = 0x054C;
pub const DUALSENSE_PIDS: [u16; 2] = [0x0CE6, 0x0DF2];

/// The subset of DualSense state we forward to the virtual pad.
/// Stick axes are raw (0..=255, 128 centered, Y grows downward).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct DsState {
    pub lx: u8,
    pub ly: u8,
    pub rx: u8,
    pub ry: u8,
    pub l2: u8,
    pub r2: u8,
    pub dpad_up: bool,
    pub dpad_down: bool,
    pub dpad_left: bool,
    pub dpad_right: bool,
    pub square: bool,
    pub cross: bool,
    pub circle: bool,
    pub triangle: bool,
    pub l1: bool,
    pub r1: bool,
    pub l3: bool,
    pub r3: bool,
    pub create: bool,
    pub options: bool,
    pub ps: bool,
}

/// Parse one HID input report. Returns None for reports we don't understand.
pub fn parse(report: &[u8]) -> Option<DsState> {
    match report.first()? {
        // USB full layout (64 bytes) or Bluetooth compact layout (10 bytes).
        0x01 if report.len() >= 11 => Some(parse_full(&report[1..])),
        0x01 if report.len() >= 10 => Some(parse_bt_compact(&report[1..])),
        // Bluetooth extended layout: one extra sequence byte before the data.
        0x31 if report.len() >= 12 => Some(parse_full(&report[2..])),
        _ => None,
    }
}

/// Full layout: LX LY RX RY L2 R2 seq buttons0 buttons1 buttons2 ...
fn parse_full(d: &[u8]) -> DsState {
    let mut s = DsState {
        lx: d[0],
        ly: d[1],
        rx: d[2],
        ry: d[3],
        l2: d[4],
        r2: d[5],
        ..DsState::default()
    };
    apply_buttons(&mut s, d[7], d[8], d[9]);
    s
}

/// Bluetooth compact layout: LX LY RX RY buttons0 buttons1 buttons2 L2 R2
fn parse_bt_compact(d: &[u8]) -> DsState {
    let mut s = DsState {
        lx: d[0],
        ly: d[1],
        rx: d[2],
        ry: d[3],
        l2: d[7],
        r2: d[8],
        ..DsState::default()
    };
    apply_buttons(&mut s, d[4], d[5], d[6]);
    s
}

fn apply_buttons(s: &mut DsState, b0: u8, b1: u8, b2: u8) {
    // b0 low nibble is the d-pad hat: 0=N clockwise to 7=NW, 8=released.
    let hat = b0 & 0x0F;
    s.dpad_up = matches!(hat, 7 | 0 | 1);
    s.dpad_right = matches!(hat, 1 | 2 | 3);
    s.dpad_down = matches!(hat, 3 | 4 | 5);
    s.dpad_left = matches!(hat, 5 | 6 | 7);
    s.square = b0 & 0x10 != 0;
    s.cross = b0 & 0x20 != 0;
    s.circle = b0 & 0x40 != 0;
    s.triangle = b0 & 0x80 != 0;
    s.l1 = b1 & 0x01 != 0;
    s.r1 = b1 & 0x02 != 0;
    s.create = b1 & 0x10 != 0;
    s.options = b1 & 0x20 != 0;
    s.l3 = b1 & 0x40 != 0;
    s.r3 = b1 & 0x80 != 0;
    s.ps = b2 & 0x01 != 0;
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Build a 64-byte USB report with the given data bytes after the ID.
    fn usb_report(data: &[u8]) -> Vec<u8> {
        let mut r = vec![0u8; 64];
        r[0] = 0x01;
        r[1..1 + data.len()].copy_from_slice(data);
        r
    }

    #[test]
    fn usb_neutral() {
        let r = usb_report(&[128, 128, 128, 128, 0, 0, 0, 0x08, 0, 0]);
        let s = parse(&r).unwrap();
        assert_eq!(s, DsState { lx: 128, ly: 128, rx: 128, ry: 128, ..DsState::default() });
    }

    #[test]
    fn usb_sticks_and_triggers() {
        let r = usb_report(&[0, 255, 10, 200, 0x40, 0xFF, 0, 0x08, 0, 0]);
        let s = parse(&r).unwrap();
        assert_eq!((s.lx, s.ly, s.rx, s.ry), (0, 255, 10, 200));
        assert_eq!((s.l2, s.r2), (0x40, 0xFF));
    }

    #[test]
    fn usb_face_buttons() {
        let r = usb_report(&[128, 128, 128, 128, 0, 0, 0, 0xF8, 0, 0]);
        let s = parse(&r).unwrap();
        assert!(s.square && s.cross && s.circle && s.triangle);
        assert!(!s.dpad_up && !s.dpad_down && !s.dpad_left && !s.dpad_right);
    }

    #[test]
    fn usb_dpad_diagonal() {
        // Hat value 1 = north-east.
        let r = usb_report(&[128, 128, 128, 128, 0, 0, 0, 0x01, 0, 0]);
        let s = parse(&r).unwrap();
        assert!(s.dpad_up && s.dpad_right && !s.dpad_down && !s.dpad_left);
    }

    #[test]
    fn usb_shoulder_and_system_buttons() {
        let r = usb_report(&[128, 128, 128, 128, 0, 0, 0, 0x08, 0xF3, 0x01]);
        let s = parse(&r).unwrap();
        assert!(s.l1 && s.r1 && s.create && s.options && s.l3 && s.r3 && s.ps);
    }

    #[test]
    fn bt_compact() {
        let r = [0x01u8, 128, 128, 128, 128, 0x28, 0x01, 0x00, 50, 60];
        let s = parse(&r).unwrap();
        assert!(s.cross && s.l1);
        assert_eq!((s.l2, s.r2), (50, 60));
    }

    #[test]
    fn bt_extended() {
        let mut r = vec![0u8; 78];
        r[0] = 0x31;
        r[2..12].copy_from_slice(&[128, 128, 128, 128, 0, 0, 0, 0x08, 0x02, 0x01]);
        let s = parse(&r).unwrap();
        assert!(s.r1 && s.ps);
    }

    #[test]
    fn unknown_report_rejected() {
        assert_eq!(parse(&[0x05, 1, 2, 3]), None);
        assert_eq!(parse(&[]), None);
        assert_eq!(parse(&[0x01, 1, 2]), None);
    }
}
