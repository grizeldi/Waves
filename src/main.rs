mod waveformwidget;

use epoxy::*;
use gtk::glib;
use gtk::prelude::{ApplicationExt, ApplicationExtManual, GtkWindowExt};
use log::{debug, info};
use std::ptr;
use crate::waveformwidget::WaveformWidget;

fn main() -> glib::ExitCode {
    env_logger::init();
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
        .build();
    application.connect_startup(|app| {
        let window = gtk::ApplicationWindow::builder()
            .application(app)
            .title("Waves")
            .default_width(1280)
            .default_height(720)
            .build();
        let waveform_display = WaveformWidget::new();
        window.set_child(Some(&waveform_display));
        window.present();
    });
    application.run();

    glib::ExitCode::SUCCESS
}
