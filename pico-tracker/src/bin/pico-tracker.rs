#![no_std]
#![no_main]

extern crate alloc;

use bevy::prelude::*;
use embedded_alloc::LlffHeap as Heap;
use embedded_graphics::{
    mono_font::{MonoTextStyle, ascii::FONT_6X12},
    pixelcolor::Rgb565,
    prelude::{Point, RgbColor},
    text::Text,
};
use hal::entry;
// use picocalc_bevy::PicoCalcDefaultPlugins;
use embedded_graphics::Drawable;
use pico_tracker_types::FromHost;
use picocalc_bevy::{Display, KeyPresses, LoggingEnv as Log, Visible, keys::KEY_ENTER};
use picocalc_tracker_lib::{
    CmdPallet, FirstViewTrack, Track, TrackID,
    base_plugin::{BasePlugin, MidiEnv},
    embedded::{Shape, TextComponent},
    exit, hal,
};

// pub use picocalc_bevy::hal;

// Tell the Boot ROM about our application
#[unsafe(link_section = ".start_block")]
#[used]
pub static IMAGE_DEF: hal::block::ImageDef = hal::block::ImageDef::secure_exe();

#[global_allocator]
static HEAP: Heap = Heap::empty();
const HEAP_SIZE: usize = 128 * 1024;

#[derive(Resource, Clone, Copy, Eq, PartialEq, PartialOrd, Ord, Deref, DerefMut)]
pub struct Playing(pub bool);

#[derive(Component, Clone, Copy, Eq, PartialEq, PartialOrd, Ord)]
pub struct PlayingMarker;

#[derive(Component, Clone, Copy, Eq, PartialEq, PartialOrd, Ord)]
pub struct DevDisplay;

#[entry]
fn main() -> ! {
    init_heap();

    App::new()
        .add_plugins(BasePlugin)
        .insert_resource(CmdPallet(false))
        .insert_resource(Playing(false))
        .init_resource::<FirstViewTrack>()
        .add_systems(Startup, (setup, screen_test))
        .add_systems(Update, (toggle_playing.run_if(enter_pressed), display_devs))
        .add_systems(PostUpdate, render)
        .run();

    // loop {}
    exit()
}

fn setup(mut cmds: Commands) {
    cmds.spawn((TrackID(0), Track::default()));
    cmds.spawn((TrackID(1), Track::default()));
    cmds.spawn((TrackID(2), Track::default()));
    cmds.spawn((TrackID(3), Track::default()));
}

fn screen_test(mut cmds: Commands, playing: Res<Playing>) {
    cmds.spawn((
        TextComponent {
            text: format!("{}", playing.0),
            point: Point::new(0, 10),
            ..default()
        },
        PlayingMarker,
    ));

    cmds.spawn((
        TextComponent {
            text: format!("{:?}", Vec::<String>::default()),
            point: Point::new(0, 20),
            ..default()
        },
        DevDisplay,
    ));
}

fn enter_pressed(keys: Res<KeyPresses>) -> bool {
    keys.just_pressed(KEY_ENTER)
}

fn toggle_playing(
    mut midi: EventWriter<MidiEnv>,
    mut log: EventWriter<Log>,
    mut text_dis: Single<&mut TextComponent, With<PlayingMarker>>,
    mut playing: ResMut<Playing>,
) {
    playing.0 = !playing.0;
    text_dis.set_text(format!("{}", playing.0));

    if playing.0 {
        midi.write(MidiEnv::On { note: 48, vel: 120 });
        log.write(Log::info("playing"));
    } else {
        midi.write(MidiEnv::Off { note: 48 });
        log.write(Log::info("not playing"));
    }
}

fn display_devs(
    mut devs: EventReader<FromHost>,
    mut text_comps: Single<(&mut TextComponent,), (With<DevDisplay>, Without<Shape>)>,
) {
    for dev_ev in devs.read() {
        // match dev_ev {
        //     FromHost::MidiNoteOn
        // }
        if let FromHost::Devs { dev_names } = dev_ev {
            text_comps.0.set_text(format!("{dev_names:?}"));
        }
    }
}

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

    let mut style = MonoTextStyle::new(&FONT_6X12, Rgb565::GREEN);
    style.background_color = Some(Rgb565::BLACK);

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

        if let Some(text) = text.old.clone()
            && (vis.is_none() || vis.as_ref().is_some_and(|vis| vis.should_show()))
        {
            let mut style = style.clone();
            style.background_color = None;
            style.text_color = Some(Rgb565::BLACK);

            // let text: String = text
            //     .text
            //     .chars()
            //     .map(|c| if !c.is_whitespace() { ' ' } else { c })
            //     .collect();
            Text::new(&text, point, style).draw(display).unwrap();
        }

        text.was_rendered();

        if vis.is_none() || vis.as_ref().is_some_and(|vis| vis.should_show()) {
            // let text = text.text.clone();
            Text::new(&text.text, point, style).draw(display).unwrap();
        } else if vis.as_ref().is_some_and(|vis| vis.should_rm()) {
            let mut style = style.clone();
            style.background_color = None;
            style.text_color = Some(Rgb565::BLACK);

            // let text: String = text
            //     .text
            //     .chars()
            //     .map(|c| if !c.is_whitespace() { ' ' } else { c })
            //     .collect();
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
