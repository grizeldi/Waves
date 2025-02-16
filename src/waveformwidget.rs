use std::cell::Cell;
use epoxy::{BindBuffer, BindVertexArray, BufferData, EnableVertexAttribArray, GenBuffers, GenVertexArrays, VertexAttribPointer, ARRAY_BUFFER, ELEMENT_ARRAY_BUFFER, FALSE, FLOAT, INT, STATIC_DRAW, UNSIGNED_INT};
use epoxy::types::{GLsizeiptr, GLuint};
use gtk::glib;
use gtk::glib::property::PropertyGet;
use gtk::subclass::prelude::ObjectSubclassIsExt;
use log::debug;
use crate::openglutils::Vertex;

mod imp {
    use crate::openglutils::{Triangle, Vertex};
    use epoxy::types::GLuint;
    use epoxy::{BindFramebuffer, BindTexture, BlitFramebuffer, Clear, ClearColor, DeleteBuffers, DeleteFramebuffers, DeleteTextures, DeleteVertexArrays, FramebufferTexture2D, GenFramebuffers, GenTextures, GetIntegerv, TexStorage2DMultisample, COLOR_ATTACHMENT0, COLOR_BUFFER_BIT, DRAW_FRAMEBUFFER, DRAW_FRAMEBUFFER_BINDING, FRAMEBUFFER, NEAREST, RGBA8, TEXTURE_2D_MULTISAMPLE};
    use gtk::gdk::GLContext;
    use gtk::glib;
    use gtk::glib::Propagation;
    use gtk::prelude::WidgetExt;
    use gtk::subclass::prelude::*;
    use log::{debug, error, trace};
    use std::cell::{Cell, RefCell};
    use crate::waveformwidget::WaveformMesh;

    #[derive(Default, Debug)]
    pub struct WaveformWidget {
        //OpenGL Handles
        offscreen_framebuffer_handle: Cell<GLuint>,
        offscreen_texture_handle: Cell<GLuint>,
        original_framebuffer_handle: Cell<GLuint>,

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
                offscreen_framebuffer_handle: Cell::new(0),
                offscreen_texture_handle: Cell::new(0),
                original_framebuffer_handle: Cell::new(0),
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
            debug!("Initializing OpenGL off screen frame buffer for WaveformWidget.");

            unsafe {
                // Create the off screen buffer
                GenFramebuffers(1, self.offscreen_framebuffer_handle.as_ptr());
                if self.offscreen_framebuffer_handle.get() == 0 {
                    error!("Failed to create off screen frame buffer!");
                    return;
                }

                // Get the handle of the main buffer
                let mut original_handle= -1;
                GetIntegerv(DRAW_FRAMEBUFFER_BINDING, &mut original_handle);
                self.original_framebuffer_handle.set(original_handle as GLuint);
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
        }
    }

    impl GLAreaImpl for WaveformWidget {
        fn render(&self, _context: &GLContext) -> Propagation {
            trace!("WaveformWidget::render");
            unsafe {
                BindFramebuffer(FRAMEBUFFER, self.offscreen_framebuffer_handle.get());

                ClearColor(0.15, 0.155, 0.17, 1.0);
                Clear(COLOR_BUFFER_BIT);

                BindFramebuffer(DRAW_FRAMEBUFFER, self.original_framebuffer_handle.get());
                BlitFramebuffer(0, 0, self.obj().width(), self.obj().height(),
                                0, 0, self.obj().width(), self.obj().height(),
                                COLOR_BUFFER_BIT, NEAREST);
                BindFramebuffer(FRAMEBUFFER, self.original_framebuffer_handle.get());
            }
            Propagation::Stop
        }

        fn resize(&self, width: i32, height: i32) {
            debug!("Resizing WaveformWidget to {}x{}px.", width, height);

            if self.offscreen_framebuffer_handle.get() == 0 {
                error!("Cannot resize waveform texture as the framebuffer doesn't exist.");
                return;
            }

            unsafe {
                if self.offscreen_texture_handle.get() != 0 {
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
            }

            self.obj().generate_mesh(width / 2, true);
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
            .build();
        waveform_widget
    }

    pub fn set_audio_file() {
        todo!()
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

            if self.imp().waveform_mesh_render_data.vao_handle.get() > 0 {
                debug!("Removing old mesh from GPU memory.");
                unsafe {
                    //TODO clean up old data or find a way to reuse existing OpenGL objects
                }
            }
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
            indices.push([vertices.len() - 1, vertices.len() - 2, vertices.len() - 3]);
            indices.push([vertices.len() - 2, vertices.len() - 3, vertices.len() - 4]);
            ids.push(i);
            ids.push(i);
        }

        // Upload to the GPU
        unsafe {
            // Allocate buffers
            GenVertexArrays(1, self.imp().waveform_mesh_render_data.vao_handle.as_ptr());
            assert_ne!(self.imp().waveform_mesh_render_data.vao_handle.get(), 0);

            let mut buffers: [GLuint; 3] = [0, 0, 0];
            GenBuffers(3, buffers.as_mut_ptr());
            self.imp().waveform_mesh_render_data.vertex_vbo_handle.set(buffers[0]);
            self.imp().waveform_mesh_render_data.id_vbo_handle.set(buffers[1]);
            self.imp().waveform_mesh_render_data.ebo_handle.set(buffers[2]);

            // Link and upload
            BindVertexArray(self.imp().waveform_mesh_render_data.vao_handle.get());

            BindBuffer(ARRAY_BUFFER, self.imp().waveform_mesh_render_data.vertex_vbo_handle.get());
            BufferData(ARRAY_BUFFER, (size_of_val(&vertices) * vertices.len()) as isize, vertices.as_ptr().cast(), STATIC_DRAW);
            VertexAttribPointer(0, 3, FLOAT, FALSE, size_of::<Vertex>().try_into().unwrap(), 0 as *const _);
            EnableVertexAttribArray(0);

            BindBuffer(ARRAY_BUFFER, self.imp().waveform_mesh_render_data.id_vbo_handle.get());
            BufferData(ARRAY_BUFFER, (size_of_val(&ids) * ids.len()) as isize, ids.as_ptr().cast(), STATIC_DRAW);
            VertexAttribPointer(1, 1, INT, FALSE, 0, 0 as *const _);
            EnableVertexAttribArray(1);

            BindBuffer(ELEMENT_ARRAY_BUFFER, self.imp().waveform_mesh_render_data.ebo_handle.get());
            BufferData(ELEMENT_ARRAY_BUFFER, (size_of_val(&indices) * indices.len()) as isize, indices.as_ptr().cast(), STATIC_DRAW);

            BindVertexArray(0);
        }
    }
}

#[derive(Default, Debug)]
struct WaveformMesh {
    vertex_vbo_handle: Cell<GLuint>,
    id_vbo_handle: Cell<GLuint>,
    vao_handle: Cell<GLuint>,
    ebo_handle: Cell<GLuint>,
}