use crate::openglutils::Vertex;
use epoxy::types::GLuint;
use epoxy::{BindBuffer, BindVertexArray, BufferData, EnableVertexAttribArray, GenBuffers, GenVertexArrays, VertexAttribIPointer, VertexAttribPointer, ARRAY_BUFFER, ELEMENT_ARRAY_BUFFER, FALSE, FLOAT, INT, STATIC_DRAW};
use gtk::glib;
use gtk::glib::property::PropertySet;
use gtk::prelude::{GLAreaExt, WidgetExt};
use gtk::subclass::prelude::ObjectSubclassIsExt;
use log::debug;
use std::cell::{Cell, RefCell};
use std::sync::{Arc, Mutex};
use std::thread;
use waves::{read_flac, separate_audio_file_into_bands, FILENAME_HIGH_BAND, FILENAME_LOW_BAND, FILENAME_MID_BAND};

const SUBDIVISION_DIVISOR: i32 = 2;

mod imp {
    use crate::openglutils::*;
    use crate::waveformwidget::{WaveformAudioData, WaveformMesh, SUBDIVISION_DIVISOR};
    use epoxy::types::{GLint, GLsizei, GLuint, GLvoid};
    use epoxy::{AttachShader, BindBuffer, BindBufferBase, BindFramebuffer, BindTexture, BindVertexArray, BlitFramebuffer, BufferData, BufferSubData, Clear, ClearColor, CompileShader, CreateProgram, CreateShader, DeleteBuffers, DeleteFramebuffers, DeleteTextures, DeleteVertexArrays, DrawElements, FramebufferTexture2D, GenBuffers, GenFramebuffers, GenTextures, GetIntegerv, LinkProgram, ShaderSource, TexStorage2DMultisample, Uniform1f, Uniform4fv, UseProgram, COLOR_ATTACHMENT0, COLOR_BUFFER_BIT, DRAW_FRAMEBUFFER, DRAW_FRAMEBUFFER_BINDING, DYNAMIC_DRAW, FRAMEBUFFER, NEAREST, RGBA8, SHADER_STORAGE_BUFFER, TEXTURE_2D_MULTISAMPLE, TRIANGLES, UNSIGNED_INT};
    use gtk::gdk::GLContext;
    use gtk::glib::property::PropertySet;
    use gtk::glib::Propagation;
    use gtk::prelude::{Cast, EventControllerExt, GLAreaExt, GestureDragExt, WidgetExt};
    use gtk::subclass::prelude::*;
    use gtk::{glib, EventControllerScroll, EventControllerScrollFlags, GestureDrag};
    use log::*;
    use std::cell::{Cell, RefCell};

    pub const VERTEX_SHADER: &str = include_str!("shaders/waveform.vert");
    pub const FRAGMENT_SHADER: &str = include_str!("shaders/waveform.frag");
    const WAVEFORM_COLORS: [Color; 3] = [
        [0.13, 0.31, 0.89, 1.0], // Low
        [0.95, 0.635, 0.2, 1.0], // Mid
        [0.96, 0.918, 0.84, 1.0] // High
    ];

    #[derive(Default, Debug)]
    pub struct WaveformWidget {
        // Audio Data
        pub audio: RefCell<WaveformAudioData>,

        // Dragging Data
        pub audio_offset: Cell<i32>,
        drag_start_offset: Cell<i32>,
        zero_source: RefCell<Vec<f32>>,

        //OpenGL Handles
        offscreen_framebuffer_handle: Cell<GLuint>,
        offscreen_texture_handle: Cell<GLuint>,
        original_framebuffer_handle: Cell<GLuint>,

        shader_program_handle: Cell<GLuint>,
        color_uniform_handle: Cell<GLint>,
        multiplier_uniform_handle: Cell<GLint>,
        values_buffer_handle: Cell<GLuint>,

        //Mesh Data
        pub waveform_mesh_render_data: WaveformMesh,
        pub waveform_mesh_vertices: RefCell<Vec<Vertex>>,
        pub waveform_mesh_indices: RefCell<Vec<Triangle>>,
        pub waveform_mesh_ids: RefCell<Vec<i32>>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for WaveformWidget {
        const NAME: &'static str = "WaveformWidget";
        type Type = super::WaveformWidget;
        type ParentType = gtk::GLArea;

        fn new() -> Self {
            Self {
                audio: Default::default(),

                audio_offset: Default::default(),
                drag_start_offset: Default::default(),
                zero_source: RefCell::new(Vec::new()),

                offscreen_framebuffer_handle: Cell::new(0),
                offscreen_texture_handle: Cell::new(0),
                original_framebuffer_handle: Cell::new(0),

                shader_program_handle: Cell::new(0),
                color_uniform_handle: Cell::new(0),
                multiplier_uniform_handle: Cell::new(0),
                values_buffer_handle: Cell::new(0),

                waveform_mesh_render_data: WaveformMesh::default(),
                waveform_mesh_vertices: RefCell::new(Vec::new()),
                waveform_mesh_indices: RefCell::new(Vec::new()),
                waveform_mesh_ids: RefCell::new(Vec::new()),
            }
        }
    }

    impl ObjectImpl for WaveformWidget {}

    impl WidgetImpl for WaveformWidget {
        fn realize(&self) {
            self.parent_realize();

            // Input handling
            let drag_handler = GestureDrag::new();
            drag_handler.connect_drag_begin(|gesture_drag: &GestureDrag, _x, _y| {
                let widget = gesture_drag.widget().unwrap();
                let global_waveform  = widget.downcast_ref::<crate::waveformwidget::WaveformWidget>().unwrap();
                let waveform : &WaveformWidget = global_waveform.imp();
                waveform.drag_start_offset.set(waveform.audio_offset.get());
            });
            drag_handler.connect_drag_update(|gesture_drag: &GestureDrag, x_offset, _y_offset| {
                let widget = gesture_drag.widget().unwrap();
                let global_waveform  = widget.downcast_ref::<crate::waveformwidget::WaveformWidget>().unwrap();
                let waveform : &WaveformWidget = global_waveform.imp();
                waveform.audio_offset.set(waveform.drag_start_offset.get() + x_offset as i32 / SUBDIVISION_DIVISOR);
                global_waveform.queue_render();
            });

            let scroll_handler = EventControllerScroll::new(EventControllerScrollFlags::VERTICAL);
            scroll_handler.connect_scroll(|scroll_handler: &EventControllerScroll, x: f64, y: f64| {
                let widget = scroll_handler.widget().unwrap();
                let global_waveform  = widget.downcast_ref::<crate::waveformwidget::WaveformWidget>().unwrap();
                global_waveform.change_zoom_level(y as i32 * 50);
                global_waveform.queue_render();
                Propagation::Stop
            });

            self.obj().add_controller(scroll_handler);
            self.obj().add_controller(drag_handler);

            // OpenGL render setup
            debug!("Initializing OpenGL off screen frame buffer for WaveformWidget.");

            unsafe {
                // Create the off-screen buffer
                GenFramebuffers(1, self.offscreen_framebuffer_handle.as_ptr());
                if self.offscreen_framebuffer_handle.get() == 0 {
                    error!("Failed to create off screen frame buffer!");
                    return;
                }

                // Prepare shaders
                debug!("Preparing waveform shaders.");
                let vert_handle = CreateShader(epoxy::VERTEX_SHADER);
                ShaderSource(vert_handle, 1, &(VERTEX_SHADER.as_bytes().as_ptr().cast()), &(VERTEX_SHADER.len().try_into().unwrap()));
                CompileShader(vert_handle);
                if !check_shader_compilation_errors(vert_handle) {
                    panic!("Vertex shader failed to compile.");
                }

                let frag_handle = CreateShader(epoxy::FRAGMENT_SHADER);
                ShaderSource(frag_handle, 1, &(FRAGMENT_SHADER.as_bytes().as_ptr().cast()), &(FRAGMENT_SHADER.len().try_into().unwrap()));
                CompileShader(frag_handle);
                if !check_shader_compilation_errors(frag_handle) {
                    panic!("Fragment shader failed to compile.");
                }

                let program_handle = CreateProgram();
                AttachShader(program_handle, vert_handle);
                AttachShader(program_handle, frag_handle);
                LinkProgram(program_handle);
                if !check_shader_linking_errors(program_handle) {
                    panic!("Shaders failed to link.");
                }
                self.shader_program_handle.set(program_handle);

                self.color_uniform_handle.set(fetch_uniform_location("renderColor", program_handle));
                self.multiplier_uniform_handle.set(fetch_uniform_location("multiplier", program_handle));

                // Create values SSBO
                let mut ssbo_handle = 0;
                GenBuffers(1, &mut ssbo_handle);
                self.values_buffer_handle.set(ssbo_handle);
            }
        }

        fn unrealize(&self) {
            self.parent_unrealize();
            trace!("WaveformWidget::unrealize");

            if self.offscreen_framebuffer_handle.get() != 0 {
                unsafe {DeleteFramebuffers(1, self.offscreen_framebuffer_handle.as_ptr());}
            }
            if self.offscreen_texture_handle.get() != 0 {
                unsafe {DeleteTextures(1, self.offscreen_texture_handle.as_ptr());}
            }
            if self.waveform_mesh_render_data.id_vbo_handle.get() != 0 {
                unsafe {DeleteBuffers(1, self.waveform_mesh_render_data.id_vbo_handle.as_ptr());}
            }
            if self.waveform_mesh_render_data.ebo_handle.get() != 0 {
                unsafe {DeleteBuffers(1, self.waveform_mesh_render_data.ebo_handle.as_ptr());}
            }
            if self.waveform_mesh_render_data.vertex_vbo_handle.get() != 0 {
                unsafe {DeleteBuffers(1, self.waveform_mesh_render_data.vertex_vbo_handle.as_ptr());}
            }
            if self.waveform_mesh_render_data.vao_handle.get() != 0 {
                unsafe {DeleteVertexArrays(1, self.waveform_mesh_render_data.vao_handle.as_ptr());}
            }
            if self.values_buffer_handle.get() != 0 {
                unsafe {DeleteBuffers(1, self.values_buffer_handle.as_ptr());}
            }
        }
    }

    impl GLAreaImpl for WaveformWidget {
        fn render(&self, _context: &GLContext) -> Propagation {
            trace!("WaveformWidget::render");
            unsafe {
                BindFramebuffer(FRAMEBUFFER, self.offscreen_framebuffer_handle.get());

                ClearColor(0.15, 0.155, 0.17, 1.0);
                Clear(COLOR_BUFFER_BIT);

                UseProgram(self.shader_program_handle.get());
                BindVertexArray(self.waveform_mesh_render_data.vao_handle.get());
                BindBuffer(SHADER_STORAGE_BUFFER, self.values_buffer_handle.get());
                BindBufferBase(SHADER_STORAGE_BUFFER, 0, self.values_buffer_handle.get());

                let audio_offset = self.audio_offset.get();
                // Draw bands
                for i in 0..3 {
                    // Set up uniforms for this band
                    Uniform4fv(self.color_uniform_handle.get(), 1, WAVEFORM_COLORS[i].as_ptr());
                    Uniform1f(self.multiplier_uniform_handle.get(), 1.0); //If we ever get around to adding gain, this is the place
                    let audio = self.audio.borrow();

                    // Handle zero padding
                    let borrowed_audio = &audio.reduced_audio[i].borrow();
                    let lower_bound = 0 - audio_offset;
                    let upper_bound = self.obj().width() / SUBDIVISION_DIVISOR - audio_offset;
                    if lower_bound >= 0 {
                        if upper_bound < borrowed_audio.len() as i32 {
                            // No padding needed
                            let data = &borrowed_audio[lower_bound as usize..upper_bound as usize];
                            BufferData(SHADER_STORAGE_BUFFER, (size_of::<f32>() * data.len()) as isize, data.as_ptr().cast(), DYNAMIC_DRAW);
                        } else {
                            // Padding at the end needed
                            let overflow_count = upper_bound as usize - borrowed_audio.len();
                            let actual_data = &borrowed_audio[lower_bound as usize..borrowed_audio.len()];
                            BufferSubData(SHADER_STORAGE_BUFFER, 0,(size_of::<f32>() * actual_data.len()) as isize, actual_data.as_ptr().cast());
                            BufferSubData(SHADER_STORAGE_BUFFER, (size_of::<f32>() * actual_data.len()) as isize, (overflow_count * size_of::<f32>()) as isize, self.zero_source.borrow().as_ptr().cast());
                        }
                    } else {
                        // Padding at the start needed
                        let underflow_count = -lower_bound as usize;
                        let actual_data = &borrowed_audio[0..upper_bound as usize];
                        BufferSubData(SHADER_STORAGE_BUFFER, 0,(size_of::<f32>() * underflow_count) as isize, self.zero_source.borrow().as_ptr().cast());
                        BufferSubData(SHADER_STORAGE_BUFFER, (size_of::<f32>() * underflow_count) as isize, (actual_data.len() * size_of::<f32>()) as isize, actual_data.as_ptr().cast());
                    }

                    DrawElements(TRIANGLES, (self.waveform_mesh_indices.borrow().len() * 3) as GLsizei, UNSIGNED_INT, 0 as *const GLvoid);
                    Uniform1f(self.multiplier_uniform_handle.get(), -1.0);
                    DrawElements(TRIANGLES, (self.waveform_mesh_indices.borrow().len() * 3) as GLsizei, UNSIGNED_INT, 0 as *const GLvoid);
                }
                BindBuffer(SHADER_STORAGE_BUFFER, 0);

                // Blit the results back to the main frame buffer
                BindFramebuffer(DRAW_FRAMEBUFFER, self.original_framebuffer_handle.get() as GLuint);
                BlitFramebuffer(0, 0, self.obj().width(), self.obj().height(),
                                0, 0, self.obj().width(), self.obj().height(),
                                COLOR_BUFFER_BIT, NEAREST);
                BindFramebuffer(FRAMEBUFFER, self.original_framebuffer_handle.get() as GLuint);
            }
            trace!("Rendering done.");
            Propagation::Stop
        }

        fn resize(&self, width: i32, height: i32) {
            self.parent_resize(width, height);
            debug!("Resizing WaveformWidget to {}x{}px.", width, height);

            if self.offscreen_framebuffer_handle.get() == 0 {
                error!("Cannot resize waveform texture as the framebuffer doesn't exist.");
                return;
            }

            unsafe {
                // Save the original buffer
                let mut original_handle= -1;
                GetIntegerv(DRAW_FRAMEBUFFER_BINDING, &mut original_handle);
                self.original_framebuffer_handle.set(original_handle as GLuint);

                if self.offscreen_texture_handle.get() != 0 {
                    debug!("Deleting existing texture.");
                    DeleteTextures(1, self.offscreen_texture_handle.as_ptr());
                }

                GenTextures(1, self.offscreen_texture_handle.as_ptr());
                if self.offscreen_texture_handle.get() == 0 {
                    error!("Failed to create off screen texture!");
                    return;
                }

                BindTexture(TEXTURE_2D_MULTISAMPLE, self.offscreen_texture_handle.get());
                TexStorage2DMultisample(TEXTURE_2D_MULTISAMPLE, 8, RGBA8, width, height, 1);
                BindFramebuffer(FRAMEBUFFER, self.offscreen_framebuffer_handle.get());
                FramebufferTexture2D(FRAMEBUFFER, COLOR_ATTACHMENT0, TEXTURE_2D_MULTISAMPLE, self.offscreen_texture_handle.get(), 0);
                BindFramebuffer(FRAMEBUFFER, self.original_framebuffer_handle.get());

                // Resize the values buffer
                debug!("Resizing the zero padding source array to contain {} zeroes.", width / SUBDIVISION_DIVISOR);
                let mut zeroes = Vec::with_capacity((width / SUBDIVISION_DIVISOR) as usize);
                for _ in 0..(width / SUBDIVISION_DIVISOR) {
                    zeroes.push(0f32);
                }

                BindBuffer(SHADER_STORAGE_BUFFER, self.values_buffer_handle.get());
                BufferData(SHADER_STORAGE_BUFFER, (size_of::<f32>() * zeroes.len()) as isize, zeroes.as_ptr().cast(), DYNAMIC_DRAW);
                BindBuffer(SHADER_STORAGE_BUFFER, 0);

                self.zero_source.set(zeroes);
            }

            self.obj().generate_mesh(width / SUBDIVISION_DIVISOR, true);
        }
    }

    impl WaveformWidget {
        pub fn reset_drag(&self) {
            self.drag_start_offset.set(0);
            self.audio_offset.set(0);
        }
    }
}

glib::wrapper! {
    pub struct WaveformWidget(ObjectSubclass<imp::WaveformWidget>)
    @extends gtk::GLArea, gtk::Widget,
    @implements gtk::Accessible, gtk::Buildable, gtk::ConstraintTarget;
}

impl WaveformWidget {
    pub fn new() -> Self {
        let waveform_widget = glib::Object::builder::<WaveformWidget>()
            .property("has-depth-buffer", false)
            .property("has-stencil-buffer", false)
            .build();
        waveform_widget
    }

    pub fn set_audio_file(&self, path: &str) {
        let audio_data = WaveformAudioData::new(path);
        audio_data.set_reduction_factor(1000); //TODO calculate this so everything fits on screen
        self.imp().reset_drag();
        self.imp().audio.set(audio_data);
        self.queue_render();
    }

    fn generate_mesh(&self, subdivisions: i32, regenerate_if_exists: bool) {
        debug!("Generating waveform mesh with {} horizontal subdivisions.", subdivisions);
        let mut vertices = self.imp().waveform_mesh_vertices.borrow_mut();
        let mut indices = self.imp().waveform_mesh_indices.borrow_mut();
        let mut ids = self.imp().waveform_mesh_ids.borrow_mut();
        if vertices.len() > 0 {
            if !regenerate_if_exists {
                return;
            }
            debug!("Waveform mesh already exists, regenerating.");
            vertices.clear();
            indices.clear();
            ids.clear();
        }

        // Generate the new mesh data
        let x_interval = 2.0 / subdivisions as f32;
        vertices.push([-1.0, 1.0, 0.0]);
        vertices.push([-1.0, 0.0, 0.0]);
        ids.push(0);
        ids.push(0);
        for i in 1..subdivisions {
            let fi = i as f32;
            vertices.push([-1.0 + fi * x_interval, 1.0, 0.0]);
            vertices.push([-1.0 + fi * x_interval, 0.0, 0.0]);
            indices.push([(vertices.len() - 1) as u32, (vertices.len() - 2) as u32, (vertices.len() - 3) as u32]);
            indices.push([(vertices.len() - 2) as u32, (vertices.len() - 3) as u32, (vertices.len() - 4) as u32]);
            ids.push(i);
            ids.push(i);
        }

        // Upload to the GPU
        unsafe {
            // Allocate buffers if they don't exist yet
            if self.imp().waveform_mesh_render_data.vao_handle.get() == 0 {
                GenVertexArrays(1, self.imp().waveform_mesh_render_data.vao_handle.as_ptr());
                assert_ne!(self.imp().waveform_mesh_render_data.vao_handle.get(), 0);

                // Assumes that if VAO doesn't exist, neither do mesh buffers
                let mut buffers: [GLuint; 3] = [0, 0, 0];
                GenBuffers(3, buffers.as_mut_ptr());
                self.imp().waveform_mesh_render_data.vertex_vbo_handle.set(buffers[0]);
                self.imp().waveform_mesh_render_data.id_vbo_handle.set(buffers[1]);
                self.imp().waveform_mesh_render_data.ebo_handle.set(buffers[2]);
            } else {
                debug!("Old VAO detected, reusing mesh buffers.");
            }

            // Link and upload
            BindVertexArray(self.imp().waveform_mesh_render_data.vao_handle.get());

            BindBuffer(ARRAY_BUFFER, self.imp().waveform_mesh_render_data.vertex_vbo_handle.get());
            BufferData(ARRAY_BUFFER, (size_of::<Vertex>() * vertices.len()) as isize, vertices.as_ptr().cast(), STATIC_DRAW);
            VertexAttribPointer(0, 3, FLOAT, FALSE, size_of::<Vertex>().try_into().unwrap(), 0 as *const _);
            EnableVertexAttribArray(0);

            BindBuffer(ARRAY_BUFFER, self.imp().waveform_mesh_render_data.id_vbo_handle.get());
            BufferData(ARRAY_BUFFER, (size_of::<i32>() * ids.len()) as isize, ids.as_ptr().cast(), STATIC_DRAW);
            VertexAttribIPointer(1, 1, INT, size_of::<i32>().try_into().unwrap(), 0 as *const _);
            EnableVertexAttribArray(1);

            BindBuffer(ELEMENT_ARRAY_BUFFER, self.imp().waveform_mesh_render_data.ebo_handle.get());
            BufferData(ELEMENT_ARRAY_BUFFER, (size_of_val(&indices) * indices.len()) as isize, indices.as_ptr().cast(), STATIC_DRAW);

            BindVertexArray(0);
            BindBuffer(ARRAY_BUFFER, 0);
            BindBuffer(ELEMENT_ARRAY_BUFFER, 0);
        }
    }

    pub fn change_zoom_level(&self, zoom_level: i32) {
        let audio = self.imp().audio.borrow_mut();
        let center_offset = self.width() / SUBDIVISION_DIVISOR / 2;
        let absolute_offset = (self.imp().audio_offset.get() - center_offset) * audio.reduction_factor.get() as i32;
        audio.set_reduction_factor((audio.reduction_factor.get() as i32 + zoom_level) as u32);
        self.imp().audio_offset.set((absolute_offset as f32 / audio.reduction_factor.get() as f32).round() as i32 + center_offset);
    }
}

#[derive(Default, Debug)]
pub struct WaveformMesh {
    vertex_vbo_handle: Cell<GLuint>,
    id_vbo_handle: Cell<GLuint>,
    vao_handle: Cell<GLuint>,
    ebo_handle: Cell<GLuint>,
}

#[derive(Debug, Default)]
pub struct WaveformAudioData {
    raw_audio: [Vec<f32>; 3],
    pub reduced_audio: [RefCell<Vec<f32>>; 3],
    reduction_factor: Cell<u32>,
}

impl WaveformAudioData {
    pub fn new(path_to_read: &str) -> Self {
        separate_audio_file_into_bands(path_to_read); //TODO implement this without relying on external ffmpeg

        // This is severely stupid. All in the name of memory safety which I can guarantee myself...
        let low_mutex = Arc::new(Mutex::new(Vec::new()));
        let low_mutex_thread = low_mutex.clone();
        let mid_mutex = Arc::new(Mutex::new(Vec::new()));
        let mid_mutex_thread = mid_mutex.clone();
        let high_mutex = Arc::new(Mutex::new(Vec::new()));
        let high_mutex_thread = high_mutex.clone();

        let low_thread = thread::spawn(move || {
            let audio_data = read_flac(FILENAME_LOW_BAND);
            let mut borrowed = low_mutex_thread.lock().unwrap();
            for sample in audio_data {
                borrowed.push(sample);
            }
        });
        let mid_thread = thread::spawn(move || {
            let audio_data = read_flac(FILENAME_MID_BAND);
            let mut borrowed = mid_mutex_thread.lock().unwrap();
            for sample in audio_data {
                borrowed.push(sample);
            }
        });
        let high_thread = thread::spawn(move || {
            let audio_data = read_flac(FILENAME_HIGH_BAND);
            let mut borrowed = high_mutex_thread.lock().unwrap();
            for sample in audio_data {
                borrowed.push(sample);
            }
        });

        // Wait for threads to complete
        low_thread.join().unwrap();
        mid_thread.join().unwrap();
        high_thread.join().unwrap();

        let out = Self {
            raw_audio: [
                Arc::try_unwrap(low_mutex).unwrap().into_inner().unwrap(),
                Arc::try_unwrap(mid_mutex).unwrap().into_inner().unwrap(),
                Arc::try_unwrap(high_mutex).unwrap().into_inner().unwrap(),
            ],
            reduction_factor: Cell::new(100),
            ..Default::default()
        };
        out.recalculate_reduced();
        out
    }

    pub fn set_reduction_factor(&self, factor: u32) {
        if factor <= 0 {
            return;
        }
        self.reduction_factor.set(factor);
        self.recalculate_reduced();
    }

    fn recalculate_reduced(&self) {
        let mut output_low = self.reduced_audio[0].borrow_mut();
        let mut output_mid = self.reduced_audio[1].borrow_mut();
        let mut output_high = self.reduced_audio[2].borrow_mut();

        output_low.clear();
        output_mid.clear();
        output_high.clear();

        for i in (0..self.raw_audio[0].len()).step_by(self.reduction_factor.get() as usize) {
            let mut range_end = i + self.reduction_factor.get() as usize;
            if range_end >= self.raw_audio[0].len() {
                range_end = self.raw_audio[0].len();
            }
            output_low.push(Self::calculate_max(&self.raw_audio[0][i..range_end]));
            output_mid.push(Self::calculate_max(&self.raw_audio[1][i..range_end]));
            output_high.push(Self::calculate_max(&self.raw_audio[2][i..range_end]));
        }
    }

    fn calculate_max(samples : &[f32]) -> f32 {
        let mut max : f32 = 0.0;
        for sample in samples {
            if (*sample).abs() > max.abs() {
                max = *sample;
            }
        }
        max.abs()
    }
}