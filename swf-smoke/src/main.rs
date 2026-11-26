use ctru::prelude::*;

use std::collections::BTreeMap;

fn is_readable_script_string(value: &str) -> bool {
    !value.is_empty()
        && value.chars().all(|character| {
            character.is_ascii_graphic() || character == ' '
        })
}

fn classify_script_string(value: &str) -> &'static str {
    let lower = value.to_ascii_lowercase();

    if lower.contains("onmouse")
        || lower.contains("onpress")
        || lower.contains("onrelease")
        || lower.contains("onroll")
        || lower.contains("onenterframe")
        || lower.contains("onkeypress")
        || lower.contains("onkeydown")
        || lower.contains("onkeyup")
        || lower.contains("onload")
        || lower.contains("onunload")
        || lower.contains("ondrag")
    {
        "Event handlers"
    } else if lower.contains("mouse")
        || lower.contains("key")
        || lower.contains("stage")
        || lower.contains("selection")
    {
        "Input and display APIs"
    } else if lower.contains("gotoand")
        || lower == "play"
        || lower == "stop"
        || lower.contains("nextframe")
        || lower.contains("prevframe")
        || lower.contains("attachmovie")
        || lower.contains("duplicatemovieclip")
        || lower.contains("removemovieclip")
        || lower.contains("loadmovie")
    {
        "Timeline and MovieClip APIs"
    } else {
        "Other readable strings"
    }
}

fn count_action_bytes(
    data: &[u8],
    counts: &mut BTreeMap<String, usize>,
    parse_errors: &mut usize,
    strings: &mut BTreeMap<String, usize>,
    trace: &mut Vec<String>,
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
                let lower_debug = debug.to_ascii_lowercase();
                let terms = [
                    "onmousedown", "onmousemove", "onmouseup", "onpress",
                    "onrelease", "onreleaseoutside", "onrollover", "onrollout",
                    "onenterframe", "_xmouse", "_ymouse", "gotoandplay",
                    "gotoandstop", "attachmovie", "removemovieclip",
                    "calclaunch", "calcpower", "operatecannon",
                    "handlecollisions", "checkterraincollisions", "moveraft",
                ];
                if terms.iter().any(|term| lower_debug.contains(term)) && trace.len() < 5000 {
                    trace.push(debug.clone());
                }

                match action {
                    swf::avm1::types::Action::Push(values) => {
                        for value in values {
                            if let Ok(swf::avm1::types::Value::Str(value)) = value {
                                *strings.entry(value.to_string_lossy(swf::UTF_8)).or_insert(0) += 1;
                            }
                        }
                    }
                    swf::avm1::types::Action::ConstantPool(pool) => {
                        for value in pool.strings {
                            *strings.entry(value.to_string_lossy(swf::UTF_8)).or_insert(0) += 1;
                        }
                    }
                    swf::avm1::types::Action::DefineFunction(function) => {
                        count_action_bytes(function.actions, counts, parse_errors, strings, trace);
                    }
                    swf::avm1::types::Action::DefineFunction2(function) => {
                        count_action_bytes(function.actions, counts, parse_errors, strings, trace);
                    }
                    swf::avm1::types::Action::With(block) => {
                        count_action_bytes(block.actions, counts, parse_errors, strings, trace);
                    }
                    swf::avm1::types::Action::Try(block) => {
                        count_action_bytes(block.try_body, counts, parse_errors, strings, trace);
                        if let Some((_, catch_body)) = block.catch_body {
                            count_action_bytes(catch_body, counts, parse_errors, strings, trace);
                        }
                        if let Some(finally_body) = block.finally_body {
                            count_action_bytes(finally_body, counts, parse_errors, strings, trace);
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
    trace: &mut Vec<String>,
) {
    for tag in tags {
        match tag {
            swf::Tag::DoAction(actions) => {
                *action_blocks += 1;
                sources.timeline_blocks += 1;
                count_action_bytes(actions, counts, parse_errors, strings, trace);
            }
            swf::Tag::DoInitAction { action_data, .. } => {
                *action_blocks += 1;
                sources.init_blocks += 1;
                count_action_bytes(action_data, counts, parse_errors, strings, trace);
            }
            swf::Tag::PlaceObject(place) => {
                if let Some(clip_actions) = &place.clip_actions {
                    for handler in &clip_actions.records {
                        *action_blocks += 1;
                        sources.clip_event_handlers += 1;
                        count_action_bytes(handler.action_data, counts, parse_errors, strings, trace);
                    }
                }
            }
            swf::Tag::DefineButton(button) | swf::Tag::DefineButton2(button) => {
                for handler in &button.actions {
                    *action_blocks += 1;
                    sources.button_event_handlers += 1;
                    count_action_bytes(handler.action_data, counts, parse_errors, strings, trace);
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
                    trace,
                );
            }
            _ => {}
        }
    }
}


fn trace_action_context(
    data: &[u8],
    source: &str,
    report: &mut Vec<String>,
    parse_errors: &mut usize,
) {
    let mut reader = swf::avm1::read::Reader::new(data);
    let mut instructions: Vec<String> = Vec::new();
    let mut nested: Vec<(String, Vec<u8>)> = Vec::new();

    while !reader.get_ref().is_empty() {
        match reader.read_action() {
            Ok(action) => {
                let debug = format!("{:?}", action);
                match &action {
                    swf::avm1::types::Action::DefineFunction(function) => {
                        nested.push((format!("{}/DefineFunction", source), function.actions.to_vec()));
                    }
                    swf::avm1::types::Action::DefineFunction2(function) => {
                        nested.push((format!("{}/DefineFunction2", source), function.actions.to_vec()));
                    }
                    swf::avm1::types::Action::With(block) => {
                        nested.push((format!("{}/With", source), block.actions.to_vec()));
                    }
                    swf::avm1::types::Action::Try(block) => {
                        nested.push((format!("{}/Try", source), block.try_body.to_vec()));
                        if let Some((_, catch_body)) = &block.catch_body {
                            nested.push((format!("{}/Catch", source), catch_body.to_vec()));
                        }
                        if let Some(finally_body) = &block.finally_body {
                            nested.push((format!("{}/Finally", source), finally_body.to_vec()));
                        }
                    }
                    _ => {}
                }
                instructions.push(debug);
                if matches!(action, swf::avm1::types::Action::End) {
                    break;
                }
            }
            Err(_) => {
                *parse_errors += 1;
                break;
            }
        }
    }

    let relevant = [
        "onmousedown", "onmousemove", "onmouseup", "onpress",
        "onrelease", "onreleaseoutside", "onrollover", "onrollout",
        "onenterframe", "_xmouse", "_ymouse", "mouselistener",
        "addlistener", "removelistener", "operatecannon", "calcpower",
        "calclaunch", "handlecollisions", "checkterraincollisions",
        "moveraft", "attachmovie", "removemovieclip",
    ];

    for (index, instruction) in instructions.iter().enumerate() {
        let lower = instruction.to_ascii_lowercase();
        if lower.starts_with("constantpool")
            && relevant.iter().any(|term| lower.contains(term))
        {
            report.push(format!("SOURCE: {}", source));
            report.push(format!("MATCH: instruction {} contains relevant ConstantPool data", index));
            let start = index.saturating_sub(5);
            let end = (index + 6).min(instructions.len());
            for context_index in start..end {
                report.push(format!("  [{}] {}", context_index, instructions[context_index]));
            }
            report.push(String::new());
        }
    }

    for (nested_source, nested_data) in nested {
        trace_action_context(&nested_data, &nested_source, report, parse_errors);
    }
}

fn trace_tag_action_context(
    tags: &[swf::Tag<'_>],
    source: &str,
    report: &mut Vec<String>,
    parse_errors: &mut usize,
) {
    for (index, tag) in tags.iter().enumerate() {
        match tag {
            swf::Tag::DoAction(actions) => {
                trace_action_context(actions, &format!("{}/DoAction#{}", source, index), report, parse_errors);
            }
            swf::Tag::DoInitAction { action_data, .. } => {
                trace_action_context(action_data, &format!("{}/DoInitAction#{}", source, index), report, parse_errors);
            }
            swf::Tag::PlaceObject(place) => {
                if let Some(clip_actions) = &place.clip_actions {
                    for (handler_index, handler) in clip_actions.records.iter().enumerate() {
                        trace_action_context(
                            handler.action_data,
                            &format!("{}/PlaceObject#{}/ClipAction#{}", source, index, handler_index),
                            report,
                            parse_errors,
                        );
                    }
                }
            }
            swf::Tag::DefineButton(button) | swf::Tag::DefineButton2(button) => {
                for (handler_index, handler) in button.actions.iter().enumerate() {
                    trace_action_context(
                        handler.action_data,
                        &format!("{}/Button#{}/Action#{}", source, index, handler_index),
                        report,
                        parse_errors,
                    );
                }
            }
            swf::Tag::DefineSprite(sprite) => {
                trace_tag_action_context(
                    &sprite.tags,
                    &format!("{}/Sprite#{}", source, index),
                    report,
                    parse_errors,
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
                            let mut action_trace = Vec::new();
                            count_tag_actions(
                                &movie.tags,
                                &mut opcode_counts,
                                &mut action_blocks,
                                &mut action_parse_errors,
                                &mut script_sources,
                                &mut string_counts,
                                &mut action_trace,
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

                            let mut readable_strings = BTreeMap::new();
                            for (value, count) in &string_counts {
                                if is_readable_script_string(value) {
                                    *readable_strings.entry(value.clone()).or_insert(0usize) += *count;
                                }
                            }

                            let mut categories: BTreeMap<&str, BTreeMap<String, usize>> = BTreeMap::new();
                            for (value, count) in &readable_strings {
                                let category = classify_script_string(value);
                                *categories
                                    .entry(category)
                                    .or_default()
                                    .entry(value.clone())
                                    .or_insert(0) += *count;
                            }

                            println!("Readable script strings: {}", readable_strings.len());
                            for category in ["Event handlers", "Input and display APIs", "Timeline and MovieClip APIs"] {
                                let entries = categories.get(category);
                                let count = entries.map_or(0, |items| items.len());
                                println!("{}: {}", category, count);
                                if let Some(items) = entries {
                                    for (value, occurrences) in items.iter().take(8) {
                                        println!("{} ({})", value, occurrences);
                                    }
                                }
                            }

                            let mut interaction_report = String::new();
                            interaction_report.push_str("Ruffle3DS Raft Wars Interaction Profile\n");
                            interaction_report.push_str(&format!("Readable unique strings: {}\n", readable_strings.len()));
                            interaction_report.push_str(&format!("Raw unique strings: {}\n\n", string_counts.len()));

                            for category in ["Event handlers", "Input and display APIs", "Timeline and MovieClip APIs", "Other readable strings"] {
                                interaction_report.push_str(&format!("{}\n", category));
                                if let Some(items) = categories.get(category) {
                                    for (value, count) in items {
                                        interaction_report.push_str(&format!("{}\t{}\n", value, count));
                                    }
                                }
                                interaction_report.push_str("\n");
                            }

                            match std::fs::write("/ruffle3ds/interaction-report.txt", interaction_report) {
                                Ok(()) => println!("Interaction report saved: PASS"),
                                Err(error) => println!("Interaction report saved: FAIL ({})", error),
                            }

                            let mut context_trace = Vec::new();
                            let mut context_parse_errors = 0;
                            trace_tag_action_context(
                                &movie.tags,
                                "Root",
                                &mut context_trace,
                                &mut context_parse_errors,
                            );
                            let mut trace_report = String::from("Ruffle3DS AVM1 Instruction Context Report\n");
                            trace_report.push_str("This report shows decoded instruction windows around relevant ConstantPool instructions.\n");
                            trace_report.push_str("It is static analysis, not runtime execution or proof of assignment/call behavior.\n");
                            trace_report.push_str(&format!("Context windows: {}\\n", context_trace.iter().filter(|line| line.starts_with("MATCH:")).count()));
                            trace_report.push_str(&format!("Nested parse errors: {}\\n\\n", context_parse_errors));
                            for line in &context_trace {
                                trace_report.push_str(line);
                                trace_report.push('\\n');
                            }
                            match std::fs::write("/ruffle3ds/action-context-report.txt", trace_report) {
                                Ok(()) => println!("Action context report saved: PASS"),
                                Err(error) => println!("Action context report saved: FAIL ({})", error),
                            }

                            let mut report = String::new();
                            report.push_str("Ruffle3DS AVM1 Opcode Inventory\n");
                            report.push_str("File: raftwars.swf\n");
                            report.push_str(&format!("Action blocks scanned: {}\n", action_blocks));
                            report.push_str(&format!("Instructions decoded: {}\n", total_instructions));
                            report.push_str(&format!("Action parse errors: {}\n\n", action_parse_errors));
                            report.push_str(&format!("Timeline blocks: {}\n", script_sources.timeline_blocks));
                            report.push_str(&format!("Init blocks: {}\n", script_sources.init_blocks));
                            report.push_str(&format!("Clip event handlers: {}\n", script_sources.clip_event_handlers));
                            report.push_str(&format!("Button event handlers: {}\n\n", script_sources.button_event_handlers));
                            report.push_str("Opcode,Count\n");

                            for (name, count) in &opcode_counts {
                                report.push_str(&format!("{},{}\n", name, count));
                            }
                            report.push_str("\nString\tCount\n");
                            for (value, count) in &string_counts {
                                let sanitized = value
                                    .replace('\n', " ")
                                    .replace('\r', " ")
                                    .replace('\t', " ");
                                report.push_str(&format!("{}\t{}\n", sanitized, count));
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
