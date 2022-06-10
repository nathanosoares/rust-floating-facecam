use pixels::{Pixels, SurfaceTexture};
use std::cmp::{max, min};
use winit::dpi::LogicalSize;
use winit::{
    event::*,
    event_loop::{ControlFlow, EventLoop},
    window::WindowBuilder,
};

const WIDTH: u32 = 300;
const HEIGHT: u32 = 300;
const BORDER: u32 = 5;
struct FrameTranform {
    x_offset: u32,
    y_offset: u32,

    widht: u32,
    height: u32,
}

impl FrameTranform {
    fn new(camera: &nokhwa::Camera) -> Self {
        Self {
            x_offset: 0,
            y_offset: 0,
            widht: camera.resolution().x(),
            height: camera.resolution().y(),
        }
    }

    fn left(&mut self) {
        if self.x_offset > 0 {
            self.x_offset(self.x_offset - 1);
        }
    }

    fn right(&mut self) {
        self.x_offset(self.x_offset + 1);
    }

    fn up(&mut self) {
        if self.y_offset > 0 {
            self.y_offset(self.y_offset - 1);
        }
    }

    fn down(&mut self) {
        self.y_offset(self.y_offset + 1);
    }

    fn x_offset(&mut self, offset: u32) {
        self.x_offset = max(0, min(self.widht - min(self.widht, HEIGHT), offset));
    }

    fn y_offset(&mut self, offset: u32) {
        self.y_offset = max(0, min(self.height - min(self.height, HEIGHT), offset));
    }

    fn center(&mut self) {
        self.x_offset((self.widht / 2) - (min(self.widht, WIDTH) / 2));
        self.y_offset((self.height / 2) - (min(self.height, HEIGHT) / 2));
    }

    fn render(&self, camera: &mut nokhwa::Camera, paint_frame: &mut [u8]) {
        let camera_frame = camera.frame().unwrap();

        for (i, pixel) in paint_frame.chunks_exact_mut(4).enumerate() {
            let x = (i % WIDTH as usize) as u32;
            let y = (i / WIDTH as usize) as u32;

            if !self.inside(WIDTH / 2, WIDTH / 2, WIDTH / 2, x, y) {
                continue;
            }

            if !self.inside(WIDTH / 2, WIDTH / 2, (WIDTH / 2) - BORDER, x, y) {
                pixel.copy_from_slice(&[0x82, 0x57, 0xE5, 0xff]);
                continue;
            }

            if x >= camera_frame.width() || y >= camera_frame.height() {
                continue;
            }

            let [r, g, b] = camera_frame
                .get_pixel(x + self.x_offset, y + self.y_offset)
                .0;

            pixel.copy_from_slice(&[r, g, b, 0xff]);
        }
    }

    fn inside(&self, circle_x: u32, circle_y: u32, rad: u32, x: u32, y: u32) -> bool {
        let circle_x = circle_x as i32;
        let circle_y = circle_y as i32;

        let x = x as i32;
        let y = y as i32;

        let rad = rad as i32;

        (x - circle_x) * (x - circle_x) + (y - circle_y) * (y - circle_y) <= rad * rad
    }
}

pub async fn run() {
    let event_loop = EventLoop::new();

    let window = {
        let size = LogicalSize::new(WIDTH as f64, HEIGHT as f64);

        WindowBuilder::new()
            .with_decorations(false)
            .with_transparent(true)
            .with_always_on_top(true)
            .with_resizable(false)
            .with_title("A fantastic window!")
            .with_inner_size(size)
            .with_min_inner_size(size)
            .build(&event_loop)
            .unwrap()
    };

    let mut camera = nokhwa::Camera::new(0, None).unwrap();

    camera.open_stream().unwrap();

    let mut pixels = {
        let window_size = window.inner_size();
        let surface_texture =
            SurfaceTexture::new(window_size.width * 2, window_size.height * 2, &window);
        Pixels::new(WIDTH, HEIGHT, surface_texture).unwrap()
    };

    pixels.set_clear_color(pixels::wgpu::Color::TRANSPARENT);

    let mut frame_tranform = FrameTranform::new(&camera);

    frame_tranform.center();

    event_loop.run(move |event, _, control_flow| {
        *control_flow = ControlFlow::Wait;

        match event {
            Event::WindowEvent { event, .. } => match event {
                WindowEvent::CloseRequested => *control_flow = ControlFlow::Exit,
                WindowEvent::MouseInput {
                    state: ElementState::Pressed,
                    button: MouseButton::Left,
                    ..
                } => window.drag_window().unwrap(),
                WindowEvent::Resized(size) => {
                    pixels.resize_surface(size.width, size.height);
                }
                WindowEvent::KeyboardInput { input, .. } => {
                    if input.state == ElementState::Pressed {
                        match input.virtual_keycode {
                            Some(VirtualKeyCode::Left) => frame_tranform.left(),
                            Some(VirtualKeyCode::Right) => frame_tranform.right(),
                            Some(VirtualKeyCode::Up) => frame_tranform.up(),
                            Some(VirtualKeyCode::Down) => frame_tranform.down(),
                            _ => (),
                        }
                    }
                }
                _ => (),
            },
            Event::RedrawRequested(_) => {
                frame_tranform.render(&mut camera, pixels.get_frame());
                if pixels
                    .render()
                    .map_err(|e| println!("pixels.render() failed: {}", e))
                    .is_err()
                {
                    *control_flow = ControlFlow::Exit;
                    return;
                }
            }
            Event::MainEventsCleared => {
                window.request_redraw();
            }
            _ => (),
        }
    });
}
