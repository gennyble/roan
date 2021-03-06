// Much of this files code is taken from the glow hello example, linked below
// https://github.com/grovesNL/glow/tree/main/examples/hello
// Comments are mostly for myself, but you might find them useful, too.
use glow::{Buffer, HasContext, Program, VertexArray};
use glutin::{
	dpi::PhysicalSize,
	event::{Event, WindowEvent},
	event_loop::{ControlFlow, EventLoop, EventLoopWindowTarget},
	window::{Window, WindowBuilder},
	ContextBuilder, ContextWrapper, GlRequest, PossiblyCurrent,
};

#[cfg(target_os = "linux")]
use glutin::platform::unix::WindowBuilderExtUnix;

// We have to keep track of two things now, so shove them into a struct.
struct Triangle {
	vao: VertexArray,
	vbo: Buffer,
}

impl Triangle {
	pub unsafe fn new(gl: &glow::Context) -> Self {
		// Vertex data in float form. Don't let rustfmt reformat
		#[rustfmt::skip]
        let verticies: [f32; 15] = [
			-0.5, -0.5, 1.0, 0.0, 0.0,
			0.5, -0.5,  0.0, 1.0, 0.0,
			0.0, 0.5,   0.0, 0.0, 1.0
		];

		// Do a weird transmute to get it into a state that buffer_data_u8_slice
		// will accept. I want buffer_data_f32_slice :(
		let vertices_u8: [u8; 60] = std::mem::transmute(verticies);

		let vao = gl.create_vertex_array().expect("Failed to create vao");
		// Bind the Vertex Array so GL knows to associate it and the buffer
		gl.bind_vertex_array(Some(vao));

		// Create a buffer, tell OpenGL it's for an array, and store some data in it.
		let vbo = gl.create_buffer().expect("Failed to create vbo");
		gl.bind_buffer(glow::ARRAY_BUFFER, Some(vbo));
		gl.buffer_data_u8_slice(glow::ARRAY_BUFFER, &vertices_u8, glow::STATIC_DRAW);

		// Our shaders has two layouts, one with two floats (vec2) for a vertex
		// and another for color (three floats; vec3).

		// The first layout (vertex location) is "(location = 0)", is 2 data_types
		// (glow::FLOAT) long and is not normalized. You can find the next vertex
		// locatation in 20 bytes (NOT data_types).
		gl.vertex_attrib_pointer_f32(0, 2, glow::FLOAT, false, 20, 0);
		gl.enable_vertex_attrib_array(0);

		// The same as above, but for color. now we're 3 glow::FLOATs long, same
		// stride, and we can be find 2 floats into the data. Notice that the last
		// argument, the offset, is 8. An f32 is 4 bytes long and there are two of
		// them before our color. Harcoding is NOT good!
		gl.vertex_attrib_pointer_f32(1, 3, glow::FLOAT, false, 20, 8);
		gl.enable_vertex_attrib_array(1);

		gl.bind_buffer(glow::ARRAY_BUFFER, None); //unbind vbo
		gl.bind_vertex_array(None); //unbind vao

		Self { vao, vbo }
	}

	// Be bad and only take a reference to ourself because borrows are hard.
	// do NOT use after delete
	//TODO: consume self
	pub unsafe fn delete(&self, gl: &glow::Context) {
		gl.delete_vertex_array(self.vao);
		gl.delete_buffer(self.vbo);
	}

	pub unsafe fn bind(&self, gl: &glow::Context) {
		gl.bind_vertex_array(Some(self.vao));
	}
}

// These two things are linked by nature. They exist together.
struct GlWindow {
	window: ContextWrapper<PossiblyCurrent, Window>,
	gl: glow::Context,
}

impl GlWindow {
	pub fn new<T>(event_loop: &EventLoopWindowTarget<T>, title: &str) -> Self {
		let window = Self::create_window(event_loop, title);

		// Get the OpenGL context.
		// What is going on here? Why's it look so weird?
		// `get_proc_address` apparently returns a pointer to an OpenGL function.
		// It's put in a closure with a &str as an argument and passed to `from_loader_function`
		// so that glow can call it with the name of every function it needs. OpenGL is weird and
		// this is how extensions are loaded I think?
		let gl = unsafe {
			glow::Context::from_loader_function(|s| window.get_proc_address(s) as *const _)
		};

		Self { window, gl }
	}

	fn create_window<T>(
		event_loop: &EventLoopWindowTarget<T>,
		title: &str,
	) -> ContextWrapper<PossiblyCurrent, Window> {
		#[cfg(target_os = "android")]
		let window_builder = WindowBuilder::new().with_title(title);

		// A quality of life thing for myself. Give the window an App ID under wayland
		// so that I can configure it to float instead of tiling with the rest of my
		// windows.
		// If you don't need this, you can remove the cfg's and just have the first
		// window_builder.
		#[cfg(target_os = "linux")]
		let window_builder = WindowBuilder::new()
			.with_title(title)
			.with_app_id("pleasefloat".into());

		// Create a new window and make it the current context.
		// unsafe is required here because `make_current` is unsafe. I believe this
		// is the case because if it fails, we don't know what state we're left in
		unsafe {
			ContextBuilder::new()
				.with_gl(GlRequest::Specific(glutin::Api::OpenGlEs, (3, 0)))
				.with_vsync(true)
				.build_windowed(window_builder, &event_loop)
				.unwrap()
				.make_current()
				.unwrap()
		}
	}
}

// A struct to hold OpenGL resources. These, because of the way OpenGl is
// handled on Android, can't persist for the lifetime of the program. Putting
// them in this struct makes them easier to manage in an Option
struct GlObjects {
	program: Program,
	triangle: Triangle,
}

// Our main application struct
struct App {
	title: String,
	running: bool,
	paused: bool,

	// Neither of these can be assumed to exist at all times because of the
	// way Android handles the OpenGL context. They have to be Options.
	glwindow: Option<GlWindow>,
	objects: Option<GlObjects>,
}

impl App {
	pub fn new(title: String) -> Self {
		Self {
			title,
			running: false,
			paused: false,
			glwindow: None,
			objects: None,
		}
	}

	pub fn run(&mut self) {
		self.running = true;

		let mut el = EventLoop::new();

		// If we're not Android, we can create our window and init OpenGL now.
		// The window on Android is created when we receive Event::Resumed
		#[cfg(not(target_os = "android"))]
		{
			self.create_window(&el);
			self.init_opengl();
		}

		while self.running {
			el.run_return(|event, event_loop, control_flow| {
				*control_flow = ControlFlow::Wait;

				self.process_events(event, event_loop, control_flow);
			});
		}
	}

	fn process_events(
		&mut self,
		event: Event<()>,
		event_loop: &EventLoopWindowTarget<()>,
		control_flow: &mut ControlFlow,
	) {
		match event {
			Event::Resumed => {
				self.paused = false;

				#[cfg(target_os = "android")]
				if ndk_glue::native_window().is_some() {
					self.create_window(event_loop);
					self.init_opengl();
				}
			}
			Event::Suspended => {
				self.paused = true;

				#[cfg(target_os = "android")]
				self.destroy_window();
			}
			Event::RedrawRequested(_window_id) => self.draw(),
			Event::WindowEvent { event, .. } => match event {
				WindowEvent::Resized(physical_size) => self.resize(physical_size),
				WindowEvent::CloseRequested => {
					if let Some(ref glw) = self.glwindow {
						let objects = self.objects.as_ref().unwrap();
						// Cleanup OpenGL things
						unsafe {
							glw.gl.delete_program(objects.program);
							objects.triangle.delete(&glw.gl);
						}
					}
					// Finally, Exit!
					*control_flow = ControlFlow::Exit;
				}
				_ => (),
			},
			_ => (),
		}
	}

	fn draw(&self) {
		if self.paused {
			return;
		}

		if let Some(ref window) = self.glwindow {
			let objects = self.objects.as_ref().unwrap();
			unsafe {
				// First we clear!
				window.gl.clear(glow::COLOR_BUFFER_BIT); // http://docs.gl/es3/glClear

				// Then we draw!
				objects.triangle.bind(&window.gl);
				window.gl.draw_arrays(glow::TRIANGLES, 0, 3);

				// Swap the buffer we just drew to with the one that's currently displayed
				window.window.swap_buffers().unwrap();
			}
		}
	}

	fn resize(&self, physical_size: PhysicalSize<u32>) {
		if let Some(ref window) = self.glwindow {
			window.window.resize(physical_size);
			unsafe {
				window.gl.viewport(
					0,
					0,
					physical_size.width as i32,
					physical_size.height as i32,
				)
			};
		}
	}

	fn create_window(&mut self, event_loop: &EventLoopWindowTarget<()>) {
		self.glwindow = Some(GlWindow::new(event_loop, &self.title));
	}

	fn destroy_window(&mut self) {
		self.glwindow = None;
	}

	// Generate all the OpenGL resources
	fn init_opengl(&mut self) {
		if let Some(ref window) = self.glwindow {
			let gl = &window.gl;

			unsafe {
				gl.clear_color(0.1, 0.5, 0.3, 1.0);

				// Create a shiny new shader program and then use it
				let program = Self::create_program(&gl);
				gl.use_program(Some(program));

				// Create vertex buffer/array
				let triangle = Triangle::new(&gl);

				self.objects = Some(GlObjects { program, triangle });
			}
		}
	}

	unsafe fn create_program(gl: &glow::Context) -> Program {
		// In OpenGL, shaders are bound to a program
		let program = gl.create_program().expect("Failed to create program");

		let shader_soruces = [
			(glow::VERTEX_SHADER, include_str!("shaders/tri.vert")),
			(glow::FRAGMENT_SHADER, include_str!("shaders/tri.frag")),
		];

		let mut shaders = vec![];
		for (stype, source) in shader_soruces.iter() {
			let shader = gl.create_shader(*stype).expect("Failed to create shader");
			gl.shader_source(shader, source);
			gl.compile_shader(shader);

			if !gl.get_shader_compile_status(shader) {
				panic!("{}", gl.get_shader_info_log(shader));
			}

			gl.attach_shader(program, shader);
			shaders.push(shader);
		}

		gl.link_program(program);
		if !gl.get_program_link_status(program) {
			panic!("{}", gl.get_program_info_log(program));
		}

		// Shaders are compiled and linked with the program, we don't need them anymore
		for shader in shaders {
			gl.detach_shader(program, shader);
			gl.delete_shader(shader);
		}

		program
	}
}

pub fn run() {
	let mut app = App::new("GlAndroid".into());
	app.run();
}
