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
use picocalc_bevy::{Display, LoggingEnv as Log, Visible};
use picocalc_tracker_lib::{
    CmdPallet, FirstViewTrack, Track, TrackID,
    base_plugin::BasePlugin,
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

#[entry]
fn main() -> ! {
    init_heap();

    App::new()
        .add_plugins(BasePlugin)
        .insert_resource(CmdPallet(false))
        .init_resource::<FirstViewTrack>()
        .add_systems(Startup, (setup,))
        .add_systems(Update, (on_update,))
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

fn screen_test(mut cmds: Commands) {
    cmds.spawn(TextComponent {
        text: "AbcdefghijklmnopqrstuvwxyzAbcdefghijklmnopqrstuvwxyzAbcdefghijklmnopqrstuvwxyz"
            .into(),
        point: Point::new(0, 10),
    });
}

fn on_update(mut logs: EventWriter<Log>) {
    logs.write(Log::info("updating"));
}

fn render(
    mut display: NonSendMut<Display>,
    // mut display: NonSendMut<DoubleFrameBuffer>,
    // mut camera: ResMut<Engine3d>,
    // mut player_buf: ResMut<DoubleBufferRes<PlayerLocation>>,
    text_comps: Query<
        (&TextComponent, Option<&mut Visible>),
        (
            Or<(Changed<TextComponent>, Changed<Visible>)>,
            Without<Shape>,
        ),
    >,
    shape_comps: Query<(Ref<Shape>, Option<&mut Visible>), Without<TextComponent>>,
    // mut logger: EventWriter<LoggingEnv>,
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

    for (text, vis) in text_comps {
        let point = text.point;

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
