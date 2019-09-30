extern crate sdl2;
extern crate nalgebra_glm as glm;
extern crate gl;

use std::ffi::{CStr, CString};

mod shader;
mod sphere;

fn main() {
    let sdl = sdl2::init().unwrap();
    let video_subsystem = sdl.video().unwrap();
    let gl_attr = video_subsystem.gl_attr();

    gl_attr.set_context_profile(sdl2::video::GLProfile::Core);
    gl_attr.set_context_version(4, 0);

    let window = video_subsystem
        .window("sphere", 512, 512)
        .opengl()
        .resizable()
        .build()
        .unwrap();

    let gl_context = window.gl_create_context().unwrap();
    let gl_ =
        gl::load_with(
            |s| video_subsystem.gl_get_proc_address(s) as *const std::os::raw::c_void
        );

    unsafe {
        gl::ClearColor(0.3, 0.3, 0.5, 1.0);
    }

    let vertex_shader = shader::Shader::from_source(
        include_str!("shaders/vertex.glsl"),
        gl::VERTEX_SHADER
    ).unwrap();

    let fragment_shader = shader::Shader::from_source(
        include_str!("shaders/fragment.glsl"),
        gl::FRAGMENT_SHADER
    ).unwrap();

    let shader_program = shader::Program::from_shaders(
        &[vertex_shader, fragment_shader]
    ).unwrap();

    let sphere = sphere::Sphere::generate(
        10.0,
        10,
        10
    );

    let mut vbo: gl::types::GLuint = 0;
    let mut ebo: gl::types::GLuint = 0;
    let mut vao: gl::types::GLuint = 0;
    unsafe {
        gl::GenBuffers(1, &mut vbo);
        gl::BindBuffer(gl::ARRAY_BUFFER, vbo);
        gl::BufferData(
            gl::ARRAY_BUFFER,
            (sphere.vertices.len() * std::mem::size_of::<f32>()) as isize,
            sphere.vertices.as_ptr() as *const gl::types::GLvoid,
            gl::STATIC_DRAW
        );

        gl::GenBuffers(1, &mut ebo);
        gl::BindBuffer(gl::ELEMENT_ARRAY_BUFFER, ebo);
        gl::BufferData(
            gl::ELEMENT_ARRAY_BUFFER,
            (sphere.indices.len() * std::mem::size_of::<i32>()) as isize,
            sphere.indices.as_ptr() as *const gl::types::GLvoid,
            gl::STATIC_DRAW
        );

        gl::GenVertexArrays(1, &mut vao);

        gl::BindVertexArray(vao);
        gl::BindBuffer(gl::ARRAY_BUFFER, vbo);

        gl::EnableVertexAttribArray(0);
        gl::VertexAttribPointer(
            0,
            3,
            gl::FLOAT,
            gl::FALSE,
            (3 * std::mem::size_of::<f32>()) as i32,
            std::ptr::null()
        );

        gl::BindBuffer(gl::ELEMENT_ARRAY_BUFFER, ebo);


        gl::BindVertexArray(0);
        gl::BindBuffer(gl::ARRAY_BUFFER, 0);
        gl::BindBuffer(gl::ELEMENT_ARRAY_BUFFER, 0);
    }

    unsafe {
        gl::Enable(gl::DEPTH_TEST);
    }

    shader_program.set_used();
    let mvp_location = unsafe {
        gl::GetUniformLocation(
            shader_program.id(),
            CString::new("mvp").unwrap().as_ptr()
        )
    };

    let mut event_pump = sdl.event_pump().unwrap();
    'main_loop: loop{
        for event in event_pump.poll_iter() {
            match event {
                sdl2::event::Event::Quit {..} => break 'main_loop,
                _ => {}
            }
        }

        unsafe {
            gl::Viewport(
                0,
                0,
                window.size().0 as i32,
                window.size().1 as i32
            );
            gl::Clear(gl::COLOR_BUFFER_BIT | gl::DEPTH_BUFFER_BIT);
        }

        let mut mvp = glm::perspective(
            1.,
            0.78,
            0.00001,
            100.
        ) * glm::look_at(
            &glm::vec3(0., 0., 30.),
            &glm::vec3(0., 0., 0.),
            &glm::vec3(0., 1., 0.)
        ) * glm::rotate(
            &glm::identity(),
            std::f32::consts::PI,
            &glm::vec3(1.0, 0., 0.0)
        );

        let mvp_raw = glm::value_ptr(&mvp);

        shader_program.set_used();
        unsafe {
            gl::UniformMatrix4fv(
                mvp_location,
                1,
                0,
                mvp_raw.as_ptr()
            );

            gl::BindVertexArray(vao);
            gl::DrawElements(
                gl::LINE_STRIP,
                sphere.indices.len() as i32,
                gl::UNSIGNED_INT,
                std::ptr::null()
            );
        }

        window.gl_swap_window();
    }
}
