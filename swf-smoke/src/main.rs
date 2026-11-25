use ctru::prelude::*;

use std::collections::BTreeMap;

fn count_action_bytes(
    data: &[u8],
    counts: &mut BTreeMap<String, usize>,
    parse_errors: &mut usize,
    strings: &mut BTreeMap<String, usize>,
) {
    let mut reader = swf::avm1::read::Reader::new(data);

    while !reader.get_ref().is_empty() {
        match reader.read_action() {
            Ok(action) => {
                let debug = format!("{:?}", action);
                let name = debug
                    .split(|character: char| character == '(' || character.is_whitespace())
                    .next()
                    .unwrap_or("Unknown")
                    .to_string();

                *counts.entry(name).or_insert(0) += 1;

                match action {
                    swf::avm1::types::Action::Push(values) => {
                        for value in values {
                            if let Ok(swf::avm1::types::Value::Str(value)) = value {
                                *strings.entry(value.to_string()).or_insert(0) += 1;
                            }
                        }
                    }
                    swf::avm1::types::Action::ConstantPool(pool) => {
                        for value in pool.strings {
                            *strings.entry(value.to_string()).or_insert(0) += 1;
                        }
                    }
                    swf::avm1::types::Action::DefineFunction(function) => {
                        count_action_bytes(function.actions, counts, parse_errors, strings);
                    }
                    swf::avm1::types::Action::DefineFunction2(function) => {
                        count_action_bytes(function.actions, counts, parse_errors, strings);
                    }
                    swf::avm1::types::Action::With(block) => {
                        count_action_bytes(block.actions, counts, parse_errors, strings);
                    }
                    swf::avm1::types::Action::Try(block) => {
                        count_action_bytes(block.try_body, counts, parse_errors, strings);
                        if let Some((_, catch_body)) = block.catch_body {
                            count_action_bytes(catch_body, counts, parse_errors, strings);
                        }
                        if let Some(finally_body) = block.finally_body {
                            count_action_bytes(finally_body, counts, parse_errors, strings);
                        }
                    }
                    swf::avm1::types::Action::End => break,
                    _ => {}
                }
            }
            Err(_) => {
                *parse_errors += 1;
                break;
            }
        }
    }
}

#[derive(Default)]
struct ScriptSources {
    timeline_blocks: usize,
    init_blocks: usize,
    clip_event_handlers: usize,
    button_event_handlers: usize,
}

fn count_tag_actions(
    tags: &[swf::Tag<'_>],
    counts: &mut BTreeMap<String, usize>,
    action_blocks: &mut usize,
    parse_errors: &mut usize,
    sources: &mut ScriptSources,
    strings: &mut BTreeMap<String, usize>,
) {
    for tag in tags {
        match tag {
            swf::Tag::DoAction(actions) => {
                *action_blocks += 1;
                sources.timeline_blocks += 1;
                count_action_bytes(actions, counts, parse_errors, strings);
            }
            swf::Tag::DoInitAction { action_data, .. } => {
                *action_blocks += 1;
                sources.init_blocks += 1;
                count_action_bytes(action_data, counts, parse_errors, strings);
            }
            swf::Tag::PlaceObject(place) => {
                if let Some(clip_actions) = &place.clip_actions {
                    for handler in &clip_actions.records {
                        *action_blocks += 1;
                        sources.clip_event_handlers += 1;
                        count_action_bytes(handler.action_data, counts, parse_errors, strings);
                    }
                }
            }
            swf::Tag::DefineButton(button) | swf::Tag::DefineButton2(button) => {
                for handler in &button.actions {
                    *action_blocks += 1;
                    sources.button_event_handlers += 1;
                    count_action_bytes(handler.action_data, counts, parse_errors, strings);
                }
            }
            swf::Tag::DefineSprite(sprite) => {
                count_tag_actions(
                    &sprite.tags,
                    counts,
                    action_blocks,
                    parse_errors,
                    sources,
                    strings,
                );
            }
            _ => {}
        }
    }
}

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

                            let mut opcode_counts = BTreeMap::new();
                            let mut action_blocks = 0;
                            let mut action_parse_errors = 0;
                            let mut script_sources = ScriptSources::default();
                            let mut string_counts = BTreeMap::new();
                            count_tag_actions(
                                &movie.tags,
                                &mut opcode_counts,
                                &mut action_blocks,
                                &mut action_parse_errors,
                                &mut script_sources,
                                &mut string_counts,
                            );

                            let total_instructions: usize = opcode_counts.values().sum();
                            println!();
                            println!("AVM1 ACTION BYTECODE");
                            println!("Action blocks scanned: {}", action_blocks);
                            println!("Instructions decoded: {}", total_instructions);
                            println!("Action parse errors: {}", action_parse_errors);
                            println!("Timeline blocks: {}", script_sources.timeline_blocks);
                            println!("Init blocks: {}", script_sources.init_blocks);
                            println!("Clip event handlers: {}", script_sources.clip_event_handlers);
                            println!("Button event handlers: {}", script_sources.button_event_handlers);

                            let mut common: Vec<_> = opcode_counts.iter().collect();
                            common.sort_by(|a, b| b.1.cmp(a.1));
                            println!("Most common instructions:");
                            for (name, count) in common.iter().take(15) {
                                println!("{}: {}", name, count);
                            }

                            let mut strings: Vec<_> = string_counts.iter().collect();
                            strings.sort_by(|a, b| b.1.cmp(a.1));
                            println!("Most common script strings:");
                            for (value, count) in strings.iter().take(30) {
                                println!("{}: {}", value, count);
                            }

                            let mut report = String::new();
                            report.push_str("Ruffle3DS AVM1 Opcode Inventory\\n");
                            report.push_str("File: raftwars.swf\\n");
                            report.push_str(&format!("Action blocks scanned: {}\\n", action_blocks));
                            report.push_str(&format!("Instructions decoded: {}\\n", total_instructions));
                            report.push_str(&format!("Action parse errors: {}\\n\\n", action_parse_errors));
                            report.push_str(&format!("Timeline blocks: {}\\n", script_sources.timeline_blocks));
                            report.push_str(&format!("Init blocks: {}\\n", script_sources.init_blocks));
                            report.push_str(&format!("Clip event handlers: {}\\n", script_sources.clip_event_handlers));
                            report.push_str(&format!("Button event handlers: {}\\n\\n", script_sources.button_event_handlers));
                            report.push_str("Opcode,Count\\n");

                            for (name, count) in &opcode_counts {
                                report.push_str(&format!("{},{}\\n", name, count));
                            }
                            report.push_str("\\nString,Count\\n");
                            for (value, count) in &string_counts {
                                let escaped = value.replace('"', "\\"");
                                report.push_str(&format!("\"{}\",{}\\n", escaped, count));
                            }

                            match std::fs::write("/ruffle3ds/avm1-opcode-report.txt", report) {
                                Ok(()) => println!("Full opcode report saved: PASS"),
                                Err(error) => println!("Full opcode report saved: FAIL ({})", error),
                            }
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
