use three_d::*;
use winit::event::WindowEvent;
use raw_window_handle::{HasRawWindowHandle, Win32WindowHandle, RawWindowHandle::Win32};

use windows::Win32::Foundation::{COLORREF, HWND};
use windows::Win32::UI::WindowsAndMessaging::{
    SetWindowLongPtrA, SetLayeredWindowAttributes,
    GWL_EXSTYLE, LWA_COLORKEY
};

// Due to a longstanding bug in Winapi's SetLayeredWindowAttributes, the R and B components of this
// colour MUST be equal.
const MAGIC_COLOUR: u32 = 0xFFB2FF;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let mut event_loop = winit::event_loop::EventLoop::new();
    let winit_window = winit::window::WindowBuilder::new()
        // .with_transparent(true)
        // .with_decorations(false)
        .with_window_level(winit::window::WindowLevel::AlwaysOnTop)
        .build(&event_loop)?;

    if let Win32(Win32WindowHandle{ hwnd, .. }) = winit_window.raw_window_handle() {
        unsafe {
            // I can't manage to coerce WS_EX_LAYERED into an isize, so magic numbers it is
            SetWindowLongPtrA(HWND(hwnd as _), GWL_EXSTYLE, 0x00080000);
            SetLayeredWindowAttributes(HWND(hwnd as _), COLORREF(MAGIC_COLOUR), 0, LWA_COLORKEY)?;
        }
    };

    let window = Window::from_winit_window(winit_window, event_loop, SurfaceSettings::default(), false)?;
    let context = window.gl();

    // Create a camera
    let mut camera = Camera::new_perspective(
        window.viewport(),
        vec3(3.0, 2.5, 6.0),
        vec3(0.0, 1.5, 0.0),
        vec3(0.0, 1.0, 0.0),
        degrees(45.0),
        0.1,
        1000.0,
    );
    let mut control = OrbitControl::new(*camera.target(), 1.0, 1000.0);

    let mut loaded = three_d_asset::io::load_async(&["modelling/loona.glb"]).await?;

    let mut cpu_model: CpuModel = loaded.deserialize("glb")?;
    cpu_model
        .geometries
        .iter_mut()
        .for_each(|part| part.compute_normals());
    let mut model = Model::<PhysicalMaterial>::new(&context, &cpu_model)?;

    let light0 = DirectionalLight::new(&context, 1.0, Srgba::WHITE, &vec3(0.0, -0.5, -0.5));
    let light1 = DirectionalLight::new(&context, 1.0, Srgba::WHITE, &vec3(0.0, 0.5, 0.5));

    let (r, g, b) = (
        ((MAGIC_COLOUR >> 16) & 0xFF) as f32 / 255.,
        ((MAGIC_COLOUR >>  8) & 0xFF) as f32 / 255.,
        ((MAGIC_COLOUR >>  0) & 0xFF) as f32 / 255.,
    );

    window.render_loop(move |mut frame_input| {
        camera.set_viewport(frame_input.viewport);
        // control.handle_events(&mut camera, &mut frame_input.events);

        // model.animate(0.001 * frame_input.accumulated_time as f32);

        frame_input
            .screen()
            .clear(ClearState::color_and_depth(r, g, b, 0., 1.0))
            .render(&camera, &model, &[&light0, &light1]);

        FrameOutput::default()
    });

    Ok(())
}
