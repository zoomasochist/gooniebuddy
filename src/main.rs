use eframe::{glow, egui_glow, egui};
use three_d::*;

const GLB_PATH: &str = "modelling/loona.glb";

fn main() {
    let native_options = eframe::NativeOptions {
        centered: true,
        renderer: eframe::Renderer::Glow,
        depth_buffer: 24,
        viewport: egui::ViewportBuilder::default()
            .with_decorations(false)
            .with_transparent(true),
        ..Default::default()
    };

    eframe::run_native("Goonie Buddy",
        native_options,
        Box::new(|cc| Box::new(MyEguiApp::new(cc)))
    );
}

#[derive(Default)]
struct MyEguiApp;

impl MyEguiApp {
    fn new(cc: &eframe::CreationContext<'_>) -> Self {
        Self::default()
    }

    fn draw_model(&mut self, ui: &mut egui::Ui) {
        let available_size = ui.available_size();
        let (rect, response) = ui.allocate_exact_size(available_size, egui::Sense::drag());

        let callback = egui::PaintCallback {
            rect,
            callback: std::sync::Arc::new(egui_glow::CallbackFn::new(move |info, painter| {
                with_three_d(painter.gl(), |three_d| {
                    three_d.frame(FrameInput::new(&three_d.context, &info, painter));
                })
            })),
        };

        ui.painter().add(callback);
    }
}

impl eframe::App for MyEguiApp {
   fn update(&mut self, ctx: &egui::Context, frame: &mut eframe::Frame) {    
        let resp =
            egui::CentralPanel::default().frame(egui::Frame::none()).show(ctx, |ui| {
                // ui.style_mut().visuals.extreme_bg_color = egui::Color32::TRANSPARENT;
                egui::Frame::canvas(ui.style()).show(ui, |ui| {
                    self.draw_model(ui);
                });
            });
        
        let response = resp.response.interact(egui::Sense::click_and_drag());

        if response.clicked_by(egui::PointerButton::Secondary) {
            println!("right clicked");
        }
   }
}

fn with_three_d<R>(gl: &std::sync::Arc<glow::Context>, f: impl FnOnce(&mut ThreeDApp) -> R) -> R {
    use std::cell::RefCell;
    thread_local! {
        pub static THREE_D: RefCell<Option<ThreeDApp>> = RefCell::new(None);
    }

    THREE_D.with(|three_d| {
        let mut three_d = three_d.borrow_mut();
        let three_d = three_d.get_or_insert_with(|| ThreeDApp::new(gl.clone()));
        f(three_d)
    })
}

pub struct FrameInput<'a> {
    screen: three_d::RenderTarget<'a>,
    viewport: three_d::Viewport,
    scissor_box: three_d::ScissorBox,
}

impl FrameInput<'_> {
    pub fn new(
        context: &three_d::Context,
        info: &egui::PaintCallbackInfo,
        painter: &egui_glow::Painter,
    ) -> Self {
        use three_d::*;

        // Constructs a screen render target to render the final image to
        let screen = painter.intermediate_fbo().map_or_else(
            || {
                RenderTarget::screen(
                    context,
                    info.viewport.width() as u32,
                    info.viewport.height() as u32,
                )
            },
            |fbo| {
                RenderTarget::from_framebuffer(
                    context,
                    info.viewport.width() as u32,
                    info.viewport.height() as u32,
                    fbo,
                )
            },
        );

        // Set where to paint
        let viewport = info.viewport_in_pixels();
        let viewport = Viewport {
            x: viewport.left_px as _,
            y: viewport.from_bottom_px as _,
            width: viewport.width_px as _,
            height: viewport.height_px as _,
        };

        // Respect the egui clip region (e.g. if we are inside an `egui::ScrollArea`).
        let clip_rect = info.clip_rect_in_pixels();
        let scissor_box = ScissorBox {
            x: clip_rect.left_px as _,
            y: clip_rect.from_bottom_px as _,
            width: clip_rect.width_px as _,
            height: clip_rect.height_px as _,
        };
        
        Self {
            screen,
            scissor_box,
            viewport,
        }
    }
}

use three_d::*;
pub struct ThreeDApp {
    context: Context,
    camera: Camera,
    model: three_d::Model<three_d::PhysicalMaterial>,
}

impl ThreeDApp {
    pub fn new(gl: std::sync::Arc<glow::Context>) -> Self {
        let context = Context::from_gl_context(gl).unwrap();

        let mut loaded = three_d_asset::io::load(&[GLB_PATH]).unwrap();
        let mut cpu_model: CpuModel = loaded.deserialize("glb").unwrap();
        
        let mut model = Model::<PhysicalMaterial>::new(&context, &cpu_model).unwrap();

        model.iter_mut().for_each(|m| m.material.render_states.cull = Cull::Back);

        let mut bounding_box = AxisAlignedBoundingBox::EMPTY;
        model.iter().for_each(|part| bounding_box.expand_with_aabb(&part.aabb()));
        println!("Model bounding box: {:?} centered at {:?}", bounding_box, bounding_box.center());

        // let camera = Camera::new_perspective(
        //     /* Viewport */ Viewport::new_at_origo(0, 0),
        //     /* Position */ bounding_box.center(),
        //     /*  Target  */ bounding_box.center(),
        //     /*    Up    */ vec3(0., 1., 0.),
        //     /*  Height  */ degrees(90.),
        //     /*  Z near  */ 0.01,
        //     /*  Z far   */ 10000.,
        // );

        let camera = Camera::new_orthographic(
            /* Viewport */ Viewport::new_at_origo(0, 0),
            /* Position */ bounding_box.center(),
            /*  Target  */ bounding_box.center(),
            /*    Up    */ vec3(0., 1., 0.),
            /*  Height  */ 10.,
            /*  Z near  */ 0.01,
            /*  Z far   */ 10000.,
        );

        Self {
            context,
            camera,
            model,
        }
    }

    fn fit_camera(&mut self, bounding_box: &AxisAlignedBoundingBox) {
        let centre = bounding_box.center();
        let size = bounding_box.size();
        let max_size = size.x.max(size.y).max(size.z);

        let up = *self.camera.up();
        let target = *self.camera.target();
        self.camera.set_view(
            vec3(max_size, max_size, max_size) + vec3(10., 10., 10.),
            target,
            up,
        );

        self.camera.set_orthographic_projection(
            100.,
            -max_size * 4.,
            max_size * 4.,
        );
    }

    // // https://discourse.threejs.org/t/camera-zoom-to-fit-object/936/2
    // fn fit_camera(&mut self, bounding_box: &AxisAlignedBoundingBox) {
    //     let mut new_pos = *self.camera.position();
    //     let fov: f32 = 90. * (std::f32::consts::PI / 180.);

    //     let centre = bounding_box.center();
    //     let size   = bounding_box.size();
    //     let max_dim = size.x.max(size.y).max(size.z);

    //     let mut camera_z = max_dim / 4. * (fov * 2.).tan();
    //     // Zoom out a little
    //     camera_z *= 1.25;
    //     new_pos.z = centre.z + camera_z;
        
    //     let up = *self.camera.up();
    //     self.camera.set_view(new_pos, centre, up);
    // }

    pub fn frame(&mut self, frame_input: FrameInput<'_>) -> Option<glow::Framebuffer> {
        let mut bounding_box = AxisAlignedBoundingBox::EMPTY;
        self.model.iter().for_each(|part| bounding_box.expand_with_aabb(&part.aabb()));

        // Ensure the viewport matches the current window viewport which changes if the window is resized
        self.camera.set_viewport(frame_input.viewport);
        self.fit_camera(&bounding_box);


        let light0 = DirectionalLight::new(&self.context, 1.0, Srgba::WHITE, &vec3(0.0, -0.5, -0.5));
        let light1 = DirectionalLight::new(&self.context, 1.0, Srgba::WHITE, &vec3(0.0, 0.5, 0.5));
    
        // Get the screen render target to be able to render something on the screen
        frame_input
            .screen
            // Clear the color and depth of the screen render target
            .clear_partially(frame_input.scissor_box, ClearState::color_and_depth(0., 0., 0., 0., 1.0))
            // Render the triangle with the color material which uses the per vertex colors defined at construction
            .render_partially(frame_input.scissor_box, &self.camera, &self.model, &[&light0, &light1]);

        frame_input.screen.into_framebuffer()
    }
}