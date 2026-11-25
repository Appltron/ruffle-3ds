use ctru::prelude::*;

const SWF_PATH: &str = "/ruffle3ds/raftwars.swf";

fn main() {
    let gfx = Gfx::new().expect("Could not initialize graphics");
    let mut hid = Hid::new().expect("Could not initialize input");
    let apt = Apt::new().expect("Could not initialize application runtime");
    let _console = Console::new(gfx.top_screen.borrow_mut());

    println!("Ruffle3DS");
    println!("Real SWF parsing test");
    println!();
    println!("Target: armv6k-nintendo-3ds");
    println!("File: {}", SWF_PATH);
    println!();

    match std::fs::read(SWF_PATH) {
        Ok(data) => {
            println!("File loading: PASS");
            println!("Bytes read: {}", data.len());

            match swf::decompress_swf(&data[..]) {
                Ok(swf_buffer) => {
                    println!("SWF decompression: PASS");
                    println!("SWF version: {}", swf_buffer.header.swf_header().version);
                    println!("Frame count: {}", swf_buffer.header.num_frames());
                    println!("Frame rate: {:?}", swf_buffer.header.frame_rate());
                    println!(
                        "ActionScript 3: {}",
                        swf_buffer.header.is_action_script_3()
                    );

                    match swf::parse_swf(&swf_buffer) {
                        Ok(movie) => {
                            println!("SWF parsing: PASS");
                            println!("Top-level tags: {}", movie.tags.len());
                        }
                        Err(error) => {
                            println!("SWF parsing: FAIL");
                            println!("{error}");
                        }
                    }
                }
                Err(error) => {
                    println!("SWF decompression: FAIL");
                    println!("{error}");
                }
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
