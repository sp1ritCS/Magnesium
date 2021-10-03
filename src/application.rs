use gtk::{gio,
          glib::{self, WeakRef},
          prelude::*,
          subclass::prelude::*};
use adw::{subclass::prelude::*, Application};
use once_cell::unsync::OnceCell;

use crate::{config,
            efg::{preferences::EfgPreferences, EfgWindow}};

mod imp {
	use super::*;

	#[derive(Default, Debug)]
	pub struct Magnesium {
		pub window: OnceCell<WeakRef<EfgWindow>>
	}

	#[glib::object_subclass]
	impl ObjectSubclass for Magnesium {
		type ParentType = Application;
		type Type = super::Magnesium;

		const NAME: &'static str = "MagnesiumApp";
	}

	impl ObjectImpl for Magnesium {}
	impl ApplicationImpl for Magnesium {
		fn activate(&self, application: &Self::Type) {
			let window = application.get_main_window();
			window.show();
			window.present();
		}

		fn startup(&self, application: &Self::Type) {
			self.parent_startup(application);
			application.set_resource_base_path(Some("/dev/sp1rit/efg/Desktop/"));

			let window = EfgWindow::new(application);
			window.set_title(Some("Magnesium"));
			self.window
				.set(window.downgrade())
				.expect("Failed to init application window");

			application.setup_actions();
			application.setup_accels();
		}
	}

	impl GtkApplicationImpl for Magnesium {}
	impl AdwApplicationImpl for Magnesium {}
}

glib::wrapper! {
	pub struct Magnesium(ObjectSubclass<imp::Magnesium>)
		@extends gio::Application, gtk::Application, Application,
		@implements gio::ActionGroup, gio::ActionMap;
}

impl Magnesium {
	pub fn new() -> Self {
		glib::Object::new(&[
			("application-id", &config::APP_ID.to_owned()),
			("flags", &gio::ApplicationFlags::empty())
		])
		.unwrap()
	}

	fn get_main_window(&self) -> EfgWindow {
		let imp = imp::Magnesium::from_instance(self);
		imp.window.get().unwrap().clone().upgrade().unwrap()
	}

	fn setup_actions(&self) {
		let quit = gio::SimpleAction::new("quit", None);
		quit.connect_activate(glib::clone!(@weak self as app => move |_, _| {
			app.quit()
		}));
		self.add_action(&quit);

		let prefs = gio::SimpleAction::new("prefs", None);
		prefs.connect_activate(glib::clone!(@weak self as app => move |_, _| {
			app.show_preferences()
		}));
		self.add_action(&prefs);

		let about = gio::SimpleAction::new("about", None);
		about.connect_activate(glib::clone!(@weak self as app => move |_, _| {
			app.show_about_diag()
		}));
		self.add_action(&about);
	}

	fn setup_accels(&self) {
		self.set_accels_for_action("app.quit", &["<Primary>q"]);
	}

	fn show_preferences(&self) {
		let win = self.get_main_window();

		let prefs = EfgPreferences::new();
		prefs.set_transient_for(Some(&win));
		prefs.set_modal(true);
		prefs.show();
	}

	fn show_about_diag(&self) {
		let win = self.get_main_window();
		let authors = vec![String::from("Florian \"sp1rit\" <sp1rit@disroot.org>")];

		let diag = gtk::AboutDialogBuilder::new()
			.authors(authors)
			.icon_name(config::APP_ID)
			.comments("Free EFG management application")
			.license_type(gtk::License::Agpl30)
			.wrap_license(true)
			.version("0.0.0-devel")
			.website("https://codeberg.org/sp1rit/magnesium/")
			.website_label("codeberg.org/sp1rit/magnesium/")
			.copyright(&format!("Copyright (c) 2021 Florian \"sp1rit\" and contributors"))
			.build();

		diag.set_transient_for(Some(&win));
		diag.set_modal(true);
		diag.show();
	}
}
