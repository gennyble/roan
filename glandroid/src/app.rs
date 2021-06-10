use glow::HasContext;
use glutin::{
    event::{Event, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
    window::WindowBuilder,
    ContextBuilder,
};

#[cfg(target_os = "linux")]
use glutin::platform::unix::WindowBuilderExtUnix;

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
    let (gl, program, vao) = unsafe {
        // Get the OpenGL context.
        // What is going on here? Why's it look so weird?
        // `get_proc_address` apparently returns a pointer to an OpenGL function.
        // It's put in a closure with a &str as an argument and passed to `from_loader_function`
        // so that glow can call it with the name of every function it needs. OpenGL is weird and
        // this is how extensions are loaded I think?
        let gl = glow::Context::from_loader_function(|s| window.get_proc_address(s) as *const _);

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

        // Use our shiny new shader program
        gl.use_program(Some(program));
        gl.clear_color(0.0, 0.0, 0.0, 1.0); // black clear color

        let vao = gl.create_vertex_array().expect("Failed to create VAO");
        gl.bind_vertex_array(Some(vao));

        (gl, program, vao)
    };

    // Main window event loop. We'll stay here for the life of the program
    event_loop.run(move |event, _, control_flow| {
        *control_flow = ControlFlow::Wait; // suspend the thread until new events arrive

        match event {
            Event::RedrawRequested(_window_id) => unsafe {
                // First we clear!
                gl.clear(glow::COLOR_BUFFER_BIT); // http://docs.gl/es3/glClear

                // Then we draw!
                gl.draw_arrays(glow::TRIANGLES, 0, 3);

                // Swap the buffer we just drew to with the one that's currently displayed
                window.swap_buffers().unwrap();
            },
            Event::WindowEvent { event, .. } => match event {
                WindowEvent::Resized(physical_size) => window.resize(physical_size),
                WindowEvent::CloseRequested => {
                    // Cleanup OpenGL things
                    unsafe {
                        gl.delete_program(program);
                        gl.delete_vertex_array(vao);
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
