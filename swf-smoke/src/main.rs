use ctru::prelude::*;

// Minimal uncompressed FWS movie:
// version 8, 1 frame, empty stage rectangle, and an End tag.
const MINIMAL_SWF: &[u8] = b"FWS\x08\x10\x00\x00\x00\x08\x00\x00\x0C\x01\x00\x00\x00";

fn main() {
    let gfx = Gfx::new().expect("Could not initialize graphics");
    let mut hid = Hid::new().expect("Could not initialize input");
    let apt = Apt::new().expect("Could not initialize application runtime");
    let _console = Console::new(gfx.top_screen.borrow_mut());

    println!("Ruffle3DS");
    println!("Upstream SWF parser test");
    println!();
    println!("Target: armv6k-nintendo-3ds");
    println!("Fixture: minimal uncompressed FWS v8");
    println!();

    let parsed = match swf::decompress_swf(MINIMAL_SWF) {
        Ok(buffer) => match swf::parse_swf(&buffer) {
            Ok(movie) => {
                println!("SWF parsing: PASS");
                println!("Parsed tags: {}", movie.tags.len());
                true
            }
            Err(error) => {
                println!("SWF parsing: FAIL");
                println!("{error}");
                false
            }
        },
        Err(error) => {
            println!("SWF decompression: FAIL");
            println!("{error}");
            false
        }
    };

    println!();
    if parsed {
        println!("Upstream SWF crate is running.");
    } else {
        println!("Upstream SWF crate could not parse fixture.");
    }
    println!("Press START to exit.");

    while apt.main_loop() {
        hid.scan_input();

        if hid.keys_down().contains(KeyPad::START) {
            break;
        }

        gfx.wait_for_vblank();
    }
}
