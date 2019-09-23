extern crate gl;

use std::ffi::{CString};

pub struct Shader {
    id: gl::types::GLuint,
}

impl Shader {
    pub fn from_source(
        source: &str,
        shader_type: gl::types::GLenum
    ) -> Result<Shader, String> {
        let id = shader_from_source(source, shader_type)?;
        return Ok(Shader{id});
    }
}

impl Drop for Shader {
    fn drop(&mut self){
        unsafe {
            gl::DeleteShader(self.id);
        }
    }
}

pub struct Program {
    id: gl::types::GLuint,
}

impl Program {
    pub fn from_shaders(shaders: &[Shader]) -> Result<Program, String> {
        let id = unsafe { gl::CreateProgram() };

        for shader in shaders {
            unsafe {
                gl::AttachShader(id, shader.id);
            }
        }

        let mut success: gl::types::GLint = 1;
        unsafe {
            gl::LinkProgram(id);
            gl::GetProgramiv(id, gl::LINK_STATUS, &mut success);
        }

        if success == 0 {
            let mut len: gl::types::GLint = 0;
            unsafe {
                gl::GetProgramiv(id, gl::INFO_LOG_LENGTH, &mut len);
            }

            let mut buffer: Vec<u8> = Vec::with_capacity(len as usize + 1);
            let error: CString = unsafe { CString::from_vec_unchecked(buffer) };

            unsafe {
                gl::GetProgramInfoLog(
                    id,
                    len,
                    std::ptr::null_mut(),
                    error.as_ptr() as *mut gl::types::GLchar
                );
            }

            return Err(error.into_string().unwrap());
        }


        for shader in shaders {
            unsafe {
                gl::DetachShader(id, shader.id);
            }
        }

        return Ok(Program{ id });
    }

    pub fn id(&self) -> gl::types::GLuint {
        self.id
    }

    pub fn set_used(&self) {
        unsafe {
            gl::UseProgram(self.id);
        }
    }
}

impl Drop for Program {
    fn drop(&mut self) {
        unsafe {
            gl::DeleteProgram(self.id);
        }
    }
}

fn shader_from_source(
    source: &str,
    shader_type: gl::types::GLuint
) -> Result<gl::types::GLuint, String> {
    let id = unsafe {gl::CreateShader(shader_type)};

    let mut success: gl::types::GLint = 1;
    unsafe {
        gl::ShaderSource(
            id,
            1,
            &CString::new(source).unwrap().as_ptr(),
            std::ptr::null());
        gl::CompileShader(id);
        gl::GetShaderiv(id, gl::COMPILE_STATUS, &mut success);
    }

    if success == 0 {
        let mut len: gl::types::GLint = 0;
        unsafe {
            gl::GetShaderiv(id, gl::INFO_LOG_LENGTH, &mut len);
        }

        let mut buffer: Vec<u8> = Vec::with_capacity(len as usize + 1);
        let error: CString = unsafe { CString::from_vec_unchecked(buffer) };

        unsafe {
            gl::GetShaderInfoLog(
                id,
                len,
                std::ptr::null_mut(),
                error.as_ptr() as *mut gl::types::GLchar
            );

            return Err(error.into_string().unwrap());
        }
    }

    return Ok(id);
}