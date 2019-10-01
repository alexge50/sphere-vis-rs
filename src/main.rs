extern crate sdl2;
extern crate nalgebra_glm as glm;
extern crate gl;
extern crate portaudio;
extern crate rustfft;

use std::ffi::{CStr, CString};
use std::sync::mpsc::channel;

use rustfft::algorithm::Radix4;
use rustfft::FFT;
use rustfft::num_complex::{Complex, Complex32};
use rustfft::num_traits::Zero;

mod shader;
mod sphere;

const SAMPLE_RATE: f64 = 44100.;
const FRAMES: u32 = 8192;
const RADIUS: f32 = 10.;
const RINGS: i32 = 10;
const SECTORS: i32 = 10;

fn wait_for_stream<F>(f: F, name: &str) -> u32
    where
        F: Fn() -> Result<portaudio::StreamAvailable, portaudio::error::Error>,
{
    'waiting_for_stream: loop {
        match f() {
            Ok(available) => match available {
                portaudio::StreamAvailable::Frames(frames) => return frames as u32,
                portaudio::StreamAvailable::InputOverflowed => println!("Input stream has overflowed"),
                portaudio::StreamAvailable::OutputUnderflowed => {
                    println!("Output stream has underflowed")
                }
            },
            Err(err) => panic!(
                "An error occurred while waiting for the {} stream: {}",
                name, err
            ),
        }
    }
}

fn main() {
    let pa = portaudio::PortAudio::new().unwrap();
    let sdl = sdl2::init().unwrap();
    let video_subsystem = sdl.video().unwrap();
    let mut timer_subsystem = sdl.timer().unwrap();
    let gl_attr = video_subsystem.gl_attr();

    println!("PortAudio");
    println!("version: {}", pa.version());
    println!("version text: {:?}", pa.version_text());
    println!("host count: {}", pa.host_api_count().unwrap());

    let default_input = pa.default_input_device().unwrap();
    let input_info = pa.device_info(default_input).unwrap();

    println!("input device: {:#?}", &input_info);

    let input_parameters = portaudio::StreamParameters::<f32>::new(
        default_input,
        1,
        true,
        input_info.default_low_input_latency
    );

    let settings = portaudio::InputStreamSettings::new(
        input_parameters,
        SAMPLE_RATE,
        FRAMES
    );

    let (sender, receiver) = channel();

    let callback = move |portaudio::InputStreamCallbackArgs {buffer, .. }| {
        match sender.send(buffer) {
            Ok(_) => portaudio::Continue,
            Err(_) => portaudio::Abort
        }
    };

    let mut stream =
        pa.open_non_blocking_stream(settings, callback).expect("Unable to create stream");

    stream.start().expect("Unable to start stream");

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
        RADIUS,
        RINGS,
        SECTORS
    );

    let mut sphere_buffer = sphere.clone();

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
            gl::DYNAMIC_DRAW
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

    let fft = Radix4::new(FRAMES as usize, false);
    let mut sound_buffer: Vec<Complex32> = [Complex::zero(); FRAMES as usize].to_vec();
    let mut output: Vec<Complex32> = [Complex::zero(); FRAMES as usize].to_vec();
    let mut frequencies: Vec<f32> = [0. as f32; FRAMES as usize / 2].to_vec();
    let mut event_pump = sdl.event_pump().unwrap();
    'main_loop: loop{
        for event in event_pump.poll_iter() {
            match event {
                sdl2::event::Event::Quit {..} => break 'main_loop,
                _ => {}
            }
        }

        let mut changed = false;
        while let Ok(buffer) = receiver.try_recv() {
            changed = true;
            for x in 0..std::cmp::min(buffer.len(), FRAMES as usize) {
                sound_buffer[x] = Complex::from(buffer[x]);
            }
        }

        if changed {
            fft.process(&mut sound_buffer, &mut output);


            for i in 0..output.len() / 2{
                frequencies[i] = (1. / SAMPLE_RATE as f32 * output[i].norm_sqr() + 1.).log10();
            }

            let mut index = 0;

            for ring in 1..RINGS {
                for sector in 0..SECTORS {
                    let p = glm::vec3(
                        sphere.vertices[(3 * (ring * SECTORS + sector)) as usize],
                        sphere.vertices[(3 * (ring * SECTORS + sector) + 1) as usize],
                        sphere.vertices[(3 * (ring * SECTORS + sector) + 2) as usize]
                    );

                    let displaced = p + glm::normalize(&p) * frequencies[index];
                    index += 1;

                    sphere_buffer.vertices[(3 * (ring * SECTORS + sector)) as usize] = displaced.x;
                    sphere_buffer.vertices[(3 * (ring * SECTORS + sector) + 1) as usize] = displaced.y;
                    sphere_buffer.vertices[(3 * (ring * SECTORS + sector) + 2) as usize] = displaced.z;
                }
            }

            unsafe {
                gl::BindBuffer(gl::ARRAY_BUFFER, vbo);
                gl::BufferSubData(
                    gl::ARRAY_BUFFER,
                    0,
                    (sphere_buffer.vertices.len() * std::mem::size_of::<f32>()) as isize,
                    sphere_buffer.vertices.as_ptr() as *const gl::types::GLvoid
                );
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
            window.size().0 as f32 / window.size().1 as f32,
            0.78,
            0.00001,
            100.
        ) * glm::look_at(
            &glm::vec3(0., 0., 40.),
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
