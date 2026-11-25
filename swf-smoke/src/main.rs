use ctru::prelude::*;

const SWF_PATH: &str = "/ruffle3ds/raftwars.swf";

fn main() {
    let gfx = Gfx::new().expect("Could not initialize graphics");
    let mut hid = Hid::new().expect("Could not initialize input");
    let apt = Apt::new().expect("Could not initialize application runtime");
    let _console = Console::new(gfx.top_screen.borrow_mut());

    println!("Ruffle3DS");
    println!("SD card file-loading test");
    println!();
    println!("Target: armv6k-nintendo-3ds");
    println!("File: {}", SWF_PATH);
    println!();

    match std::fs::read(SWF_PATH) {
        Ok(data) => {
            println!("File loading: PASS");
            println!("Bytes read: {}", data.len());

            if data.len() >= 3 {
                let signature = &data[0..3];
                println!("SWF signature: {}", String::from_utf8_lossy(signature));
            } else {
                println!("SWF signature: file is too short");
            }
        }
        Err(error) => {
            println!("File loading: FAIL");
            println!("Could not read {}:", SWF_PATH);
            println!("{error}");
            println!();
            println!("Check that the file exists in the virtual SD card.");
        }
    }

    println!();
    println!("Press START to exit.");

    while apt.main_loop() {
        hid.scan_input();

        if hid.keys_down().contains(KeyPad::START) {
            break;
        }

        gfx.wait_for_vblank();
    }
}
