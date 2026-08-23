// On non-Windows the modules only serve the test suite.
#![cfg_attr(not(windows), allow(dead_code))]

mod dualsense;
mod mapping;

#[cfg(windows)]
fn main() -> Result<(), Box<dyn std::error::Error>> {
    use dualsense::{DUALSENSE_PIDS, SONY_VID};

    let api = hidapi::HidApi::new()?;
    let info = api
        .device_list()
        .find(|d| d.vendor_id() == SONY_VID && DUALSENSE_PIDS.contains(&d.product_id()))
        .ok_or("no DualSense found - connect it via USB or Bluetooth first")?;
    let bus = info.bus_type();
    let device = info.open_device(&api)?;
    println!("dsx: found DualSense ({bus:?})");

    let client = vigem_client::Client::connect()
        .map_err(|e| format!("cannot reach ViGEmBus driver (is it installed?): {e}"))?;
    let mut target = vigem_client::Xbox360Wired::new(client, vigem_client::TargetId::XBOX360_WIRED);
    target.plugin()?;
    target.wait_ready()?;
    println!("dsx: virtual Xbox 360 pad plugged in - bridging (Ctrl+C to stop)");

    let mut buf = [0u8; 128];
    loop {
        let n = device.read_timeout(&mut buf, 1000)?;
        if n == 0 {
            continue; // timeout, controller idle or asleep
        }
        if let Some(state) = dualsense::parse(&buf[..n]) {
            let r = mapping::map(&state);
            let gamepad = vigem_client::XGamepad {
                buttons: vigem_client::XButtons { raw: r.buttons },
                left_trigger: r.left_trigger,
                right_trigger: r.right_trigger,
                thumb_lx: r.thumb_lx,
                thumb_ly: r.thumb_ly,
                thumb_rx: r.thumb_rx,
                thumb_ry: r.thumb_ry,
            };
            target.update(&gamepad)?;
        }
    }
}

#[cfg(not(windows))]
fn main() {
    eprintln!("dsx only runs on Windows (it talks to ViGEmBus and the Windows HID stack)");
    std::process::exit(1);
}
