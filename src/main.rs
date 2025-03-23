mod waveformwidget;
mod openglutils;

use epoxy::*;
use gtk::glib;
use gtk::prelude::{ApplicationExt, ApplicationExtManual, FileExt, GtkWindowExt};
use log::{debug, error, info, warn};
use std::ptr;
use gtk::gio::{ApplicationFlags, File};
use waves::cleanup;
use crate::waveformwidget::WaveformWidget;

fn main() -> glib::ExitCode {
    env_logger::builder()
        .format_timestamp_millis()
        .init();
    info!("Starting Waves.");

    debug!("Loading epoxy OpenGL functions.");
    {
        #[cfg(target_os = "macos")]
        let library = unsafe { libloading::os::unix::Library::new("libepoxy.0.dylib") }.unwrap();
        #[cfg(all(unix, not(target_os = "macos")))]
        let library = unsafe { libloading::os::unix::Library::new("libepoxy.so.0") }.unwrap();
        #[cfg(windows)]
        let library = libloading::os::windows::Library::open_already_loaded("libepoxy-0.dll")
            .or_else(|_| libloading::os::windows::Library::open_already_loaded("epoxy-0.dll"))
            .unwrap();

        load_with(|name| {
            unsafe { library.get::<_>(name.as_bytes()) }
                .map(|symbol| *symbol)
                .unwrap_or(ptr::null())
        });
    }

    debug!("Creating application window.");
    let application = gtk::Application::builder()
        .application_id("com.grizeldi.Waves")
        .flags(ApplicationFlags::HANDLES_OPEN)
        .build();
    application.connect_startup(build_ui);
    application.connect_shutdown(|_application| {
        info!("Shutting down. Goodbye.");
        cleanup();
    });
    application.run();

    glib::ExitCode::SUCCESS
}

fn build_ui(application: &gtk::Application) {
    let window = gtk::ApplicationWindow::builder()
        .application(application)
        .title("Waves")
        .default_width(1280)
        .default_height(720)
        .build();
    let waveform_display = WaveformWidget::new();
    window.set_child(Some(&waveform_display));
    application.connect_open(move |_, files : &[File], _| {
        if files.len() == 0 {
            error!("Opening zero files??");
            return;
        }
        if files.len() > 1 {
            warn!("Requested to open multiple files. This is not supported, only opening the first one.");
        }
        let file = &files[0];
        info!("Opening file {:?}", file.path().unwrap().into_os_string());
        waveform_display.set_audio_file(file.path().unwrap().to_str().unwrap())
    });
    window.present();
}