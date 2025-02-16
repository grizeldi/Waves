use std::ffi::CString;
use epoxy::{GetError, GetProgramInfoLog, GetProgramiv, GetShaderInfoLog, GetShaderiv, GetUniformLocation, COMPILE_STATUS, INFO_LOG_LENGTH, LINK_STATUS, NO_ERROR};
use epoxy::types::{GLchar, GLint, GLuint};
use log::error;

// Types
pub type Vertex = [f32; 3];
pub type Triangle = [u32; 3];
pub type Color = [f32; 4];

// Utility functions
pub unsafe fn check_shader_compilation_errors(shader_handle: GLuint) -> bool {
    let mut success = -1;
    GetShaderiv(shader_handle, COMPILE_STATUS, &mut success);
    if success != 1 {
        let mut log_length = 0;
        GetShaderiv(shader_handle, INFO_LOG_LENGTH, &mut log_length);

        let mut log = create_whitespace_cstring_with_len(log_length as usize);
        GetShaderInfoLog(shader_handle, log_length, &mut log_length, log.as_ptr() as *mut GLchar);

        let s = log.into_string().unwrap();
        error!("Shader compilation failed:\n {}", s);
        return false;
    }
    true
}

pub unsafe fn check_shader_linking_errors(program_handle: GLuint) -> bool {
    let mut success = -1;
    GetProgramiv(program_handle, LINK_STATUS, &mut success);
    if success != 1 {
        let mut log_length = 0;
        GetProgramiv(program_handle, INFO_LOG_LENGTH, &mut log_length);

        let mut log = create_whitespace_cstring_with_len(log_length as usize);
        GetProgramInfoLog(program_handle, log_length, std::ptr::null_mut(), log.as_ptr() as *mut GLchar);

        let s = log.into_string().unwrap();
        error!("Shader linking failed:\n {}", s);
        return false;
    }
    true
}

pub fn query_gl_error() {
    loop {
        let error_code = unsafe { GetError() };
        if error_code == NO_ERROR {
            break;
        }
        error!("Current GL Error: {}", error_code);
    }
}

pub fn fetch_uniform_location(name: &str, program_handle: GLuint) -> GLint {
    let uniform_name = CString::new(name).unwrap();
    unsafe {GetUniformLocation(program_handle, uniform_name.as_ptr().cast())}
}

fn create_whitespace_cstring_with_len(len: usize) -> CString {
    // allocate buffer of correct size
    let mut buffer: Vec<u8> = Vec::with_capacity(len + 1);
    // fill it with len spaces
    buffer.extend([b' '].iter().cycle().take(len));
    // convert buffer to CString
    unsafe { CString::from_vec_unchecked(buffer) }
}