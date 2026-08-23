// On non-Windows the modules only serve the test suite.
#![cfg_attr(not(windows), allow(dead_code))]

mod dualsense;
mod mapping;
mod rumble;

#[cfg(windows)]
fn main() -> Result<(), Box<dyn std::error::Error>> {
    use dualsense::{DUALSENSE_PIDS, SONY_VID};
    use std::sync::mpsc;

    let api = hidapi::HidApi::new()?;
    let info = api
        .device_list()
        .find(|d| d.vendor_id() == SONY_VID && DUALSENSE_PIDS.contains(&d.product_id()))
        .ok_or("no DualSense found - connect it via USB or Bluetooth first")?;
    let is_usb = matches!(info.bus_type(), hidapi::BusType::Usb);
    let device = info.open_device(&api)?;
    println!("dsx: found DualSense ({:?})", info.bus_type());

    let client = vigem_client::Client::connect()
        .map_err(|e| format!("cannot reach ViGEmBus driver (is it installed?): {e}"))?;
    let mut target = vigem_client::Xbox360Wired::new(client, vigem_client::TargetId::XBOX360_WIRED);
    target.plugin()?;
    target.wait_ready()?;

    // Rumble notifications arrive on their own thread; the HID device is not
    // Sync, so forward motor values to the main loop over a channel.
    let (rumble_tx, rumble_rx) = mpsc::channel::<(u8, u8)>();
    if is_usb {
        let reqn = target.request_notification()?;
        reqn.spawn_thread(move |_, n| {
            let _ = rumble_tx.send((n.large_motor, n.small_motor));
        });
        println!("dsx: rumble passthrough active");
    } else {
        println!("dsx: rumble passthrough is USB-only for now");
    }
    println!("dsx: virtual Xbox 360 pad plugged in - bridging (Ctrl+C to stop)");

    let mut buf = [0u8; 128];
    loop {
        // 50ms timeout bounds rumble latency when the input stream is idle;
        // in practice the controller streams at ~250Hz so writes are prompt.
        let n = device.read_timeout(&mut buf, 50)?;

        if let Some((large, small)) = rumble_rx.try_iter().last() {
            let _ = device.write(&rumble::usb_report(large, small));
        }

        if n == 0 {
            continue;
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
