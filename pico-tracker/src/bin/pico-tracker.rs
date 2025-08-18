#![no_std]
#![no_main]

extern crate alloc;

use bevy::prelude::*;
use embedded_alloc::LlffHeap as Heap;
use embedded_graphics::{
    mono_font::{MonoTextStyle, ascii::FONT_8X13},
    pixelcolor::Rgb565,
    prelude::{Point, RgbColor},
    text::Text,
};
use hal::entry;
// use picocalc_bevy::PicoCalcDefaultPlugins;
use embedded_graphics::Drawable;
use picocalc_bevy::{Display, KeyPresses, LoggingEnv as Log, Visible, keys::*};
use picocalc_tracker_lib::{
    CHAR_H, COL_W, CmdPallet, FirstViewTrack, Track, TrackID,
    base_plugin::{BasePlugin, MidiEnv},
    display_midi_note,
    embedded::{Shape, TextComponent},
    exit, hal, row_from_line, x_from_col,
};

// pub use picocalc_bevy::hal;

// Tell the Boot ROM about our application
#[unsafe(link_section = ".start_block")]
#[used]
pub static IMAGE_DEF: hal::block::ImageDef = hal::block::ImageDef::secure_exe();

#[global_allocator]
static HEAP: Heap = Heap::empty();
const HEAP_SIZE: usize = 256 * 1024;

#[derive(Resource, Clone, Copy, Eq, PartialEq, PartialOrd, Ord, Deref, DerefMut)]
pub struct Playing(pub bool);

#[derive(Component, Clone, Copy, Eq, PartialEq, PartialOrd, Ord)]
pub struct PlayingMarker;

#[derive(Component, Clone, Copy, Eq, PartialEq, PartialOrd, Ord)]
pub struct DevDisplay;

#[derive(Component, Clone, Copy, Eq, PartialEq, PartialOrd, Ord)]
pub struct CellMarker {
    track: u8,
    column: u8,
    row: u8,
}

#[derive(Component, Clone, Copy, Eq, PartialEq, PartialOrd, Ord)]
pub struct LineNumMarker {
    track: u8,
    row: u8,
}

#[derive(Component, Clone, Copy, Eq, PartialEq, PartialOrd, Ord, Deref, DerefMut)]
pub struct TitleMarker(pub u8);

#[derive(Component, Clone, Copy, Eq, PartialEq, PartialOrd, Ord)]
pub struct CursorText;

#[derive(Component, Clone, Copy, Eq, PartialEq, PartialOrd, Ord, Deref, DerefMut)]
pub struct CursorID(usize);

#[derive(Resource, Default, Clone, Copy, Eq, PartialEq, PartialOrd, Ord)]
pub struct CursorLocation(pub usize, pub usize);

#[entry]
fn main() -> ! {
    init_heap();

    App::new()
        .add_plugins(BasePlugin)
        .insert_resource(CmdPallet(false))
        .insert_resource(Playing(false))
        .init_resource::<FirstViewTrack>()
        .init_resource::<CursorLocation>()
        .add_systems(Startup, (setup_tracks, setup_track_dis, setup_cursor))
        .add_systems(
            Update,
            (
                toggle_playing.run_if(enter_pressed),
                display_tracks,
                cursor_tracks.run_if(not(shift_pressed)),
                display_cursor,
            ),
        )
        .add_systems(PostUpdate, render)
        .run();

    // loop {}
    exit()
}

fn setup_tracks(mut cmds: Commands) {
    cmds.spawn((TrackID(0), Track::default()));
    cmds.spawn((TrackID(1), Track::default()));
    // cmds.spawn((TrackID(2), Track::default()));
    // cmds.spawn((TrackID(3), Track::default()));
}

fn setup_cursor(mut cmds: Commands) {
    cmds.spawn((
        TextComponent {
            text: ">".into(),
            point: Point::new(x_from_col(2), row_from_line(2)),
            color: Some(Rgb565::CYAN),
            ..default()
        },
        CursorText,
    ));
}

fn setup_track_dis(mut cmds: Commands) {
    let n_col = 2;

    for col_n in 0u8..n_col {
        let x_offset = x_from_col(COL_W * col_n as usize);

        cmds.spawn((
            TextComponent {
                text: format!("Chan: {}", col_n + 1),
                point: Point::new(x_offset as i32, row_from_line(0)),
                ..default()
            },
            TitleMarker(col_n),
        ));

        for (i, line_n) in (2..CHAR_H - 1).enumerate() {
            let y_offset = row_from_line(line_n);
            let row = i as u8;

            // line number
            cmds.spawn((
                TextComponent {
                    text: format!("{: >2}", i + 1),
                    point: Point::new(x_offset as i32, y_offset),
                    ..default()
                },
                LineNumMarker { track: col_n, row },
            ));

            cmds.spawn((
                TextComponent {
                    text: ">".into(),
                    point: Point::new(x_from_col(2) + x_offset, y_offset),
                    color: Some(Rgb565::CYAN),
                    ..default()
                },
                // Visible::new(false),
                CursorID(i * 6 + col_n as usize * 3 + 0),
            ));

            // Note display
            cmds.spawn((
                TextComponent {
                    text: "---".into(),
                    point: Point::new(x_offset as i32 + x_from_col(3), y_offset),
                    ..default()
                },
                CellMarker {
                    track: col_n,
                    column: 0,
                    row,
                },
            ));

            cmds.spawn((
                TextComponent {
                    text: ">".into(),
                    // point: Point::new(x_from_col(2), row_from_line(2)),
                    point: Point::new(x_offset as i32 + x_from_col(6), y_offset),
                    color: Some(Rgb565::CYAN),
                    ..default()
                },
                // Visible::new(false),
                CursorID(i * 6 + (col_n as usize * 3) + 1),
            ));

            // cmd 1
            cmds.spawn((
                TextComponent {
                    text: "----".into(),
                    point: Point::new(x_offset as i32 + x_from_col(7), y_offset),
                    ..default()
                },
                CellMarker {
                    track: col_n,
                    column: 1,
                    row,
                },
            ));

            cmds.spawn((
                TextComponent {
                    text: ">".into(),
                    point: Point::new(x_offset + x_from_col(11), y_offset),
                    color: Some(Rgb565::CYAN),
                    ..default()
                },
                // Visible::new(false),
                CursorID(i * 6 + (col_n as usize * 3) + 2),
            ));

            // cmd 2
            cmds.spawn((
                TextComponent {
                    text: "----".into(),
                    point: Point::new(x_offset as i32 + x_from_col(12), y_offset),
                    ..default()
                },
                CellMarker {
                    track: col_n,
                    column: 2,
                    row,
                },
            ));
        }
    }
}

// fn screen_test(mut cmds: Commands, playing: Res<Playing>) {
//     cmds.spawn((
//         TextComponent {
//             text: format!("{}", playing.0).to_uppercase(),
//             point: Point::new(0, row_from_line(1)),
//             ..default()
//         },
//         PlayingMarker,
//     ));
//
//     // cmds.spawn((
//     //     TextComponent {
//     //         text: format!("{:?}", Vec::<String>::default()),
//     //         point: Point::new(0, row_from_line(1)),
//     //         ..default()
//     //     },
//     //     DevDisplay,
//     // ));
// }

fn display_tracks(
    text_comps: Query<(&mut TextComponent, &CellMarker)>,
    tracks: Query<(&Track, &TrackID)>,
) {
    let mut tracks: Vec<(&Track, &TrackID)> = tracks.into_iter().collect();
    tracks.sort_by_key(|(_track, id): &(&Track, &TrackID)| id.0);

    for (ref mut text, cell) in text_comps {
        let track = tracks[cell.track as usize].0;
        // track.
        match track {
            Track::Midi { steps } => {
                let step = steps[cell.row as usize].clone();
                text.set_text(
                    [
                        step.note
                            .map(display_midi_note)
                            .unwrap_or("---".to_string()),
                        format!("{}", step.cmds.0),
                        format!("{}", step.cmds.1),
                    ][cell.column as usize]
                        .clone(),
                );
            }
            _ => {}
        }
    }
}

fn shift_pressed(keys: Res<KeyPresses>) -> bool {
    keys.is_pressed(KEY_MOD_SHL) || keys.is_pressed(KEY_MOD_SHR)
}

fn cursor_tracks(keys: Res<KeyPresses>, mut location: ResMut<CursorLocation>) {
    let CursorLocation(x, y) = *location;

    if keys.just_pressed(KEY_UP)
        && !keys.is_pressed(KEY_DOWN)
        && !keys.is_pressed(KEY_LEFT)
        && !keys.is_pressed(KEY_RIGHT)
    {
        if y == 0 {
            location.1 = CHAR_H - 4;
        } else {
            location.1 -= 1;
        };
    } else if keys.just_pressed(KEY_DOWN)
        && !keys.is_pressed(KEY_UP)
        && !keys.is_pressed(KEY_LEFT)
        && !keys.is_pressed(KEY_RIGHT)
    {
        if y == CHAR_H - 4 {
            location.1 = 0;
        } else {
            location.1 += 1;
            location.1 %= CHAR_H - 4;
        };
    } else if keys.just_pressed(KEY_LEFT)
        && !keys.is_pressed(KEY_UP)
        && !keys.is_pressed(KEY_DOWN)
        && !keys.is_pressed(KEY_RIGHT)
    {
        if x == 0 {
            location.0 = 5;
        } else {
            location.0 -= 1;
        }
    } else if keys.just_pressed(KEY_RIGHT)
        && !keys.is_pressed(KEY_UP)
        && !keys.is_pressed(KEY_DOWN)
        && !keys.is_pressed(KEY_LEFT)
    {
        if x == 5 {
            location.0 = 0;
        } else {
            location.0 += 1;
            location.0 %= 6;
        };
    }
}

fn display_cursor(
    cursors: Query<(&mut Visible, &CursorID), With<TextComponent>>,
    loc: Res<CursorLocation>,
) {
    let target = loc.1 * 6 + loc.0;

    for (ref mut vis, CursorID(id)) in cursors {
        // if *id == target && !vis.should_show() {
        vis.set_visible(*id == target);
        // } else if *id != target && vis.should_show() {
        // vis.set_visible(false);
        // }
    }
}

fn enter_pressed(keys: Res<KeyPresses>) -> bool {
    keys.just_pressed(KEY_ENTER)
}

fn toggle_playing(
    mut midi: EventWriter<MidiEnv>,
    mut log: EventWriter<Log>,
    // mut text_dis: Single<&mut TextComponent, With<PlayingMarker>>,
    mut playing: ResMut<Playing>,
) {
    playing.0 = !playing.0;
    // text_dis.set_text(format!("{}", playing.0).to_uppercase());

    if playing.0 {
        midi.write(MidiEnv::On { note: 48, vel: 120 });
        log.write(Log::info("playing"));
    } else {
        midi.write(MidiEnv::Off { note: 48 });
        log.write(Log::info("not playing"));
    }
}

// fn display_devs(
//     mut devs: EventReader<FromHost>,
//     mut text_comps: Single<(&mut TextComponent,), (With<DevDisplay>, Without<Shape>)>,
// ) {
//     for dev_ev in devs.read() {
//         // match dev_ev {
//         //     FromHost::MidiNoteOn
//         // }
//         if let FromHost::Devs { dev_names } = dev_ev {
//             text_comps.0.set_text(format!("{dev_names:?}"));
//         }
//     }
// }

fn render(
    mut display: NonSendMut<Display>,
    text_comps: Query<
        (&mut TextComponent, Option<&mut Visible>),
        (
            Or<(Changed<TextComponent>, Changed<Visible>)>,
            Without<Shape>,
        ),
    >,
    shape_comps: Query<(Ref<Shape>, Option<&mut Visible>), Without<TextComponent>>,
) {
    let Display { output: display } = display.as_mut();

    // let cam_changed = camera.changed || player_buf.was_updated();

    // let setup_cam = |player: &PlayerLocation, camera: &mut ResMut<Engine3d>| {
    //     camera.engine.camera.set_position(player.pos);
    //     let lookat = player.looking_at;
    //     camera.engine.camera.set_target(lookat);
    // };
    //
    // for (mesh, vis) in mesh_comps {
    //     // "unrender" all meshes if changed or camera changed
    //     if cam_changed && (vis.is_none() || vis.as_ref().is_some_and(|vis| vis.should_rm())) {
    //         setup_cam(player_buf.get_inactive(), &mut camera);
    //         let mut renderable = K3dMeshe:new(Geometry {
    //             vertices: &mesh.vertices,
    //             faces: &[],
    //             colors: &[],
    //             lines: &mesh.lines,
    //             normals: &[],
    //         });
    //         renderable.set_render_mode(mesh.render_mode);
    //         renderable.set_scale(mesh.scale);
    //         renderable.set_color(Rgb565::BLACK);
    //         camera.engine.render([&renderable], |p| draw(p, display))
    //     }
    //
    //     // "rerender" a ll renderables if changed or camera changed
    //     if (vis.is_none() || vis.as_ref().is_some_and(|vis| vis.should_show()))
    //         && (mesh.is_changed() || cam_changed)
    //     {
    //         setup_cam(player_buf.get_active(), &mut camera);
    //         let mut renderable = K3dMesh::new(Geometry {
    //             vertices: &mesh.vertices,
    //             faces: &[],
    //             colors: &[],
    //             lines: &mesh.lines,
    //             normals: &[],
    //         });
    //         renderable.set_render_mode(mesh.render_mode);
    //         renderable.set_scale(mesh.scale);
    //         renderable.set_color(mesh.color);
    //         camera.engine.render([&renderable], |p| draw(p, display))
    //     }
    //
    //     vis.map(|ref mut vis| vis.was_rendered());
    // }

    let mut style = MonoTextStyle::new(&FONT_8X13, Rgb565::GREEN);
    // style.background_color = Some(Rgb565::BLACK);
    style.background_color = None;

    // for (text, vis) in text_comps.clone() {
    //     let point = text.point;
    //
    //     // if vis.is_none() || vis.as_ref().is_some_and(|vis| vis.should_show()) {
    //     //     // let text = text.text.clone();
    //     //     Text::new(&text.text, point, style).draw(display).unwrap();
    //     // } else if vis.as_ref().is_some_and(|vis| vis.should_rm()) {
    //     //     let mut style = style.clone();
    //     //     style.background_color = None;
    //     //     style.text_color = Some(Rgb565::BLACK);
    //     //
    //     //     // let text: String = text
    //     //     //     .text
    //     //     //     .chars()
    //     //     //     .map(|c| if !c.is_whitespace() { ' ' } else { c })
    //     //     //     .collect();
    //     //     Text::new(&text.text, point, style).draw(display).unwrap();
    //     // }
    //
    //     // vis.map(|ref mut vis| vis.was_rendered());
    // }

    for (ref mut text, vis) in text_comps {
        let point = text.point;

        if let Some(display_text) = text.old.clone()
            && (vis.is_none() || vis.as_ref().is_some_and(|vis| vis.should_show()))
        {
            let mut style = style.clone();
            // style.text_color = Some(Rgb565::BLACK);
            style.text_color = Some(text.bg_color.unwrap_or(Rgb565::BLACK));
            Text::new(&display_text, point, style)
                .draw(display)
                .unwrap();
        }

        text.was_rendered();

        if vis.is_none() || vis.as_ref().is_some_and(|vis| vis.should_show()) {
            // let text = text.text.clone();
            let mut style = style.clone();
            style.text_color = Some(text.color.unwrap_or(Rgb565::GREEN));
            Text::new(&text.text, point, style).draw(display).unwrap();
        } else if vis.as_ref().is_some_and(|vis| vis.should_rm()) {
            let mut style = style.clone();
            style.text_color = Some(text.bg_color.unwrap_or(Rgb565::BLACK));
            Text::new(&text.text, point, style).draw(display).unwrap();
        }

        vis.map(|ref mut vis| vis.was_rendered());
    }

    // display.frame_len(&mut logger);
    // display.draw_frame(&mut logger);

    // call DoubleBufferRes::switch()
    // player_buf.switch();
    // camera.changed = false;
}

#[allow(static_mut_refs)]
fn init_heap() {
    use core::mem::MaybeUninit;
    static mut HEAP_MEM: [MaybeUninit<u8>; HEAP_SIZE] = [MaybeUninit::uninit(); HEAP_SIZE];
    unsafe { HEAP.init(HEAP_MEM.as_ptr() as usize, HEAP_SIZE) }
}

/// Program metadata for `picotool info`
#[unsafe(link_section = ".bi_entries")]
#[used]
pub static PICOTOOL_ENTRIES: [hal::binary_info::EntryAddr; 5] = [
    hal::binary_info::rp_cargo_bin_name!(),
    hal::binary_info::rp_cargo_version!(),
    hal::binary_info::rp_program_description!(c"PicoCalc-Tracker"),
    hal::binary_info::rp_cargo_homepage_url!(),
    hal::binary_info::rp_program_build_attribute!(),
];
