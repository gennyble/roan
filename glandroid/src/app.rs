use glow::{Buffer, HasContext, Program, VertexArray};
use glutin::{
	dpi::PhysicalSize,
	event::{Event, WindowEvent},
	event_loop::{ControlFlow, EventLoop},
	window::WindowBuilder,
	ContextBuilder,
};

#[cfg(target_os = "linux")]
use glutin::platform::unix::WindowBuilderExtUnix;

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

		let vbo = gl.create_buffer().expect("Failed to create vbo");
		gl.bind_buffer(glow::ARRAY_BUFFER, Some(vbo));
		gl.buffer_data_u8_slice(glow::ARRAY_BUFFER, &vertices_u8, glow::STATIC_DRAW);

		gl.vertex_attrib_pointer_f32(0, 2, glow::FLOAT, false, 20, 0);
		gl.enable_vertex_attrib_array(0);

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

// Much of this code is taken from the glow hello example, linked below
// https://github.com/grovesNL/glow/tree/main/examples/hello
// Comments are mostly for myself, but you might find them useful, too.

pub fn run() {
	let event_loop = EventLoop::new();

	#[cfg(target_os = "android")]
	let window_builder = WindowBuilder::new().with_title("GlAndroid");

	// A quality of life thing for myself. Give the window an App ID under wayland
	// so that I can configure it to float instead of tiling with the rest of my
	// windows.
	// If you don't need this, you can remove the cfg's and just have the first
	// window_builder.
	#[cfg(target_os = "linux")]
	let window_builder = WindowBuilder::new()
		.with_title("GlAndroid")
		.with_app_id("pleasefloat".into());

	// Create a new window and make it the current context.
	// unsafe is required here because `make_current` is unsafe. I believe this
	// is the case because if it fails, we don't know what state we're left in
	let window = unsafe {
		ContextBuilder::new()
			.with_vsync(true)
			.build_windowed(window_builder, &event_loop)
			.unwrap()
			.make_current()
			.unwrap()
	};

	// unsafe land! it would be good to abstract the unsafe away, but I'd like
	// to keep this example simple. You can do it yourself, you're pretty smart!
	let (gl, program, triangle) = unsafe {
		// Get the OpenGL context.
		// What is going on here? Why's it look so weird?
		// `get_proc_address` apparently returns a pointer to an OpenGL function.
		// It's put in a closure with a &str as an argument and passed to `from_loader_function`
		// so that glow can call it with the name of every function it needs. OpenGL is weird and
		// this is how extensions are loaded I think?
		let gl = glow::Context::from_loader_function(|s| window.get_proc_address(s) as *const _);

		// Create a siny new shader program and then use it
		let program = create_program(&gl);
		gl.use_program(Some(program));

		// Create vertex buffer/array
		let triangle = Triangle::new(&gl);

		//gl.clear_color(0.1, 0.5, 0.3, 1.0);
		gl.clear_color(0.0, 0.0, 0.0, 1.0);

		(gl, program, triangle)
	};

	// Main window event loop. We'll stay here for the life of the program
	event_loop.run(move |event, _, control_flow| {
		*control_flow = ControlFlow::Wait; // suspend the thread until new events arrive

		match event {
			Event::RedrawRequested(_window_id) => unsafe {
				// First we clear!
				gl.clear(glow::COLOR_BUFFER_BIT); // http://docs.gl/es3/glClear

				// Then we draw!
				triangle.bind(&gl);
				gl.draw_arrays(glow::TRIANGLES, 0, 3);

				// Swap the buffer we just drew to with the one that's currently displayed
				window.swap_buffers().unwrap();
			},
			Event::WindowEvent { event, .. } => match event {
				WindowEvent::Resized(physical_size) => {
					window.resize(physical_size);
					unsafe {
						gl.viewport(
							0,
							0,
							physical_size.width as i32,
							physical_size.height as i32,
						)
					};
				}
				WindowEvent::CloseRequested => {
					// Cleanup OpenGL things
					unsafe {
						gl.delete_program(program);
						triangle.delete(&gl);
					}
					// Finally, Exit!
					*control_flow = ControlFlow::Exit;
				}
				_ => (),
			},
			_ => (),
		}
	});
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
