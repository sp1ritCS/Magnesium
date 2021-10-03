mod application;
pub mod config;
pub mod efg;
pub mod models;

use gtk::{gio::{prelude::*, resources_register, Resource},
          glib};

fn main() {
	gtk::init().expect("Failed to initialize GTK");

	let res =
		Resource::load(std::env::var("EFG_DEVEL_DATADIR").unwrap_or(config::PKGDATADIR.to_owned()) + "/efg.gresource")
			.expect("Failed loading resources");
	resources_register(&res);

	glib::set_application_name("Magnesium");
	glib::set_program_name(Some(&config::APP_ID));
	gtk::Window::set_default_icon_name(config::APP_ID);

	let app = application::Magnesium::new();
	std::process::exit(app.run());
}
