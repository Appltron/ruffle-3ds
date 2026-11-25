use ctru::prelude::*;

const SWF_PATH: &str = "/ruffle3ds/raftwars.swf";

#[derive(Default)]
struct TagCounts {
    shapes: usize,
    sprites: usize,
    placements: usize,
    removals: usize,
    show_frames: usize,
    actions: usize,
    bitmaps: usize,
    fonts: usize,
    text: usize,
    buttons: usize,
    sounds: usize,
    other: usize,
    total: usize,
}

fn count_tags(tags: &[swf::Tag<'_>], counts: &mut TagCounts) {
    for tag in tags {
        counts.total += 1;

        match tag {
            swf::Tag::DefineShape(_) => counts.shapes += 1,
            swf::Tag::DefineSprite(sprite) => {
                counts.sprites += 1;
                count_tags(&sprite.tags, counts);
            }
            swf::Tag::PlaceObject(_) => counts.placements += 1,
            swf::Tag::RemoveObject(_) => counts.removals += 1,
            swf::Tag::ShowFrame => counts.show_frames += 1,
            swf::Tag::DoAction(_) | swf::Tag::DoInitAction { .. } => counts.actions += 1,
            swf::Tag::DefineBits { .. }
            | swf::Tag::DefineBitsJpeg2 { .. }
            | swf::Tag::DefineBitsJpeg3(_)
            | swf::Tag::DefineBitsLossless(_) => counts.bitmaps += 1,
            swf::Tag::DefineFont(_)
            | swf::Tag::DefineFont2(_)
            | swf::Tag::DefineFont4(_)
            | swf::Tag::DefineFontInfo(_) => counts.fonts += 1,
            swf::Tag::DefineText(_) | swf::Tag::DefineText2(_) => counts.text += 1,
            swf::Tag::DefineButton(_) | swf::Tag::DefineButton2(_) => counts.buttons += 1,
            swf::Tag::DefineSound(_)
            | swf::Tag::StartSound(_)
            | swf::Tag::StartSound2 { .. }
            | swf::Tag::SoundStreamHead(_)
            | swf::Tag::SoundStreamHead2(_)
            | swf::Tag::SoundStreamBlock(_) => counts.sounds += 1,
            _ => counts.other += 1,
        }
    }
}

fn main() {
    let gfx = Gfx::new().expect("Could not initialize graphics");
    let mut hid = Hid::new().expect("Could not initialize input");
    let apt = Apt::new().expect("Could not initialize application runtime");
    let _console = Console::new(gfx.top_screen.borrow_mut());

    println!("Ruffle3DS");
    println!("SWF tag inventory test");
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

                    match swf::parse_swf(&swf_buffer) {
                        Ok(movie) => {
                            println!("SWF parsing: PASS");
                            println!("SWF version: {}", movie.header.version());
                            println!("Frames: {}", movie.header.num_frames());
                            println!("Frame rate: {:?}", movie.header.frame_rate());
                            println!("AS3: {}", movie.header.is_action_script_3());
                            println!();

                            let mut shapes = 0;
                            let mut sprites = 0;
                            let mut placements = 0;
                            let mut removals = 0;
                            let mut show_frames = 0;
                            let mut actions = 0;
                            let mut bitmaps = 0;
                            let mut fonts = 0;
                            let mut text = 0;
                            let mut buttons = 0;
                            let mut sounds = 0;

                            for tag in &movie.tags {
                                match tag {
                                    swf::Tag::DefineShape(_) => shapes += 1,
                                    swf::Tag::DefineSprite(_) => sprites += 1,
                                    swf::Tag::PlaceObject(_) => placements += 1,
                                    swf::Tag::RemoveObject(_) => removals += 1,
                                    swf::Tag::ShowFrame => show_frames += 1,
                                    swf::Tag::DoAction(_) | swf::Tag::DoInitAction { .. } => actions += 1,
                                    swf::Tag::DefineBits { .. }
                                    | swf::Tag::DefineBitsJpeg2 { .. }
                                    | swf::Tag::DefineBitsJpeg3(_)
                                    | swf::Tag::DefineBitsLossless(_) => bitmaps += 1,
                                    swf::Tag::DefineFont(_)
                                    | swf::Tag::DefineFont2(_)
                                    | swf::Tag::DefineFont4(_)
                                    | swf::Tag::DefineFontInfo(_) => fonts += 1,
                                    swf::Tag::DefineText(_) | swf::Tag::DefineText2(_) => text += 1,
                                    swf::Tag::DefineButton(_) | swf::Tag::DefineButton2(_) => buttons += 1,
                                    swf::Tag::DefineSound(_)
                                    | swf::Tag::StartSound(_)
                                    | swf::Tag::StartSound2 { .. }
                                    | swf::Tag::SoundStreamHead(_)
                                    | swf::Tag::SoundStreamHead2(_)
                                    | swf::Tag::SoundStreamBlock(_) => sounds += 1,
                                    _ => {}
                                }
                            }

                            println!("TOP-LEVEL TAG COUNTS");
                            println!("Shapes: {}", shapes);
                            println!("Sprites: {}", sprites);
                            println!("Placements: {}", placements);
                            println!("Removals: {}", removals);
                            println!("ShowFrame: {}", show_frames);
                            println!("Action blocks: {}", actions);
                            println!("Bitmaps: {}", bitmaps);
                            println!("Fonts: {}", fonts);
                            println!("Text: {}", text);
                            println!("Buttons: {}", buttons);
                            println!("Sound tags: {}", sounds);
                            println!();
                            println!("Note: counts exclude tags nested inside sprites.");

                            let mut nested = TagCounts::default();
                            for tag in &movie.tags {
                                if let swf::Tag::DefineSprite(sprite) = tag {
                                    count_tags(&sprite.tags, &mut nested);
                                }
                            }

                            println!();
                            println!("NESTED SPRITE TAG COUNTS");
                            println!("Total nested tags: {}", nested.total);
                            println!("Shapes: {}", nested.shapes);
                            println!("Sprites: {}", nested.sprites);
                            println!("Placements: {}", nested.placements);
                            println!("Removals: {}", nested.removals);
                            println!("ShowFrame: {}", nested.show_frames);
                            println!("Action blocks: {}", nested.actions);
                            println!("Bitmaps: {}", nested.bitmaps);
                            println!("Fonts: {}", nested.fonts);
                            println!("Text: {}", nested.text);
                            println!("Buttons: {}", nested.buttons);
                            println!("Sound tags: {}", nested.sounds);
                            println!("Other tags: {}", nested.other);
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
