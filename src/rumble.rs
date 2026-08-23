//! DualSense rumble output report (USB).
//!
//! Layout follows the Linux hid-playstation driver and SDL: report ID 0x02,
//! then two "valid flag" bytes, then the high-frequency (right) and
//! low-frequency (left) motor strengths. Flag bits 0+1 select classic
//! rumble emulation instead of raw haptics.

pub const USB_REPORT_LEN: usize = 48;

/// Build the USB output report for the given Xbox-style motor pair.
/// `large` is the strong low-frequency motor, `small` the weak one.
pub fn usb_report(large: u8, small: u8) -> [u8; USB_REPORT_LEN] {
    let mut r = [0u8; USB_REPORT_LEN];
    r[0] = 0x02; // report ID
    r[1] = 0x03; // COMPATIBLE_VIBRATION | HAPTICS_SELECT
    r[3] = small; // right / high-frequency motor
    r[4] = large; // left / low-frequency motor
    r
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn report_shape() {
        let r = usb_report(200, 50);
        assert_eq!(r[0], 0x02);
        assert_eq!(r[1], 0x03);
        assert_eq!(r[2], 0);
        assert_eq!(r[3], 50);
        assert_eq!(r[4], 200);
        assert!(r[5..].iter().all(|&b| b == 0));
    }

    #[test]
    fn zero_motors_stop_rumble() {
        let r = usb_report(0, 0);
        assert_eq!((r[3], r[4]), (0, 0));
        assert_eq!(r[1], 0x03); // flags stay set so the stop is applied
    }
}
