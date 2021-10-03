use sm::Schulmanager;
use gtk::{gio::{self, Settings},
          glib,
          prelude::*,
          subclass::prelude::*,
          ButtonsType,
          CompositeTemplate,
          MessageDialog,
          MessageType,
          ToggleButton};
use adw::PreferencesWindow;
use once_cell::unsync::OnceCell;

use crate::models::secret::{Secrets, ServiceProvider};

pub mod imp {
	use adw::subclass::prelude::*;
	use gtk::{Button, Entry, PasswordEntry, SpinButton};

	use super::*;
	#[derive(Debug, Default, CompositeTemplate)]
	#[template(resource = "/dev/sp1rit/efg/Desktop/ui/prefs.ui")]
	pub struct EfgPreferences {
		pub settings: OnceCell<gio::Settings>,

		#[template_child]
		pub bhvr_weekends_no: TemplateChild<ToggleButton>,
		#[template_child]
		pub bhvr_weekends_sat: TemplateChild<ToggleButton>,
		#[template_child]
		pub bhvr_weekends_yes: TemplateChild<ToggleButton>,
		#[template_child]
		pub bhvr_wkstart: TemplateChild<adw::ActionRow>,
		#[template_child]
		pub bhvr_wkstart_mon: TemplateChild<ToggleButton>,
		#[template_child]
		pub bhvr_wkstart_sun: TemplateChild<ToggleButton>,

		#[template_child]
		pub tt_blocksize: TemplateChild<SpinButton>,

		#[template_child]
		pub online_accounts: TemplateChild<adw::PreferencesPage>,

		#[template_child]
		pub o365_user: TemplateChild<Entry>,
		#[template_child]
		pub o365_pass: TemplateChild<PasswordEntry>,
		#[template_child]
		pub o365_btn: TemplateChild<Button>,
		#[template_child]
		pub o365_delete: TemplateChild<Button>
	}

	#[glib::object_subclass]
	impl ObjectSubclass for EfgPreferences {
		type ParentType = PreferencesWindow;
		type Type = super::EfgPreferences;

		const NAME: &'static str = "EfgPreferences";

		fn class_init(klass: &mut Self::Class) {
			Self::bind_template(klass);
		}

		fn instance_init(obj: &glib::subclass::InitializingObject<Self>) {
			obj.init_template();
		}
	}

	impl ObjectImpl for EfgPreferences {
		fn constructed(&self, obj: &Self::Type) {
			obj.setup_triggers();
			self.parent_constructed(obj);
		}
	}

	impl WidgetImpl for EfgPreferences {}
	impl WindowImpl for EfgPreferences {}
	impl AdwWindowImpl for EfgPreferences {}
	impl PreferencesWindowImpl for EfgPreferences {}
}

glib::wrapper! {
	pub struct EfgPreferences(ObjectSubclass<imp::EfgPreferences>)
		@extends gtk::Widget, gtk::Window, adw::PreferencesWindow,
		@implements gio::ActionMap, gio::ActionGroup;
}

impl EfgPreferences {
	pub fn new() -> Self {
		glib::Object::new(&[]).expect("Failed to create EfgPreferences")
	}

	pub fn setup_triggers(&self) {
		let ctx = glib::MainContext::default();
		let self_ = imp::EfgPreferences::from_instance(self);
		let settings = Settings::new(crate::config::APP_ID);

		// BEGIN General
		// BEGIN Behavior
		fn update_weekends(settings: &Settings, no: &ToggleButton, sat: &ToggleButton, yes: &ToggleButton) {
			match settings.enum_("bhvr-weekends") {
				0 => no.set_active(true),
				1 => sat.set_active(true),
				2 => yes.set_active(true),
				_ => eprintln!("bhvr-weekends did not contain expected value")
			}
		}
		fn update_weekstart(settings: &Settings, mon: &ToggleButton, sun: &ToggleButton) {
			match settings.enum_("bhvr-weekstart") {
				0 => mon.set_active(true),
				1 => sun.set_active(true),
				_ => eprintln!("bhvr-weekstart did not contain expected value")
			}
		}

		let bhvr_weekends_no = self_.bhvr_weekends_no.get();
		let bhvr_weekends_sat = self_.bhvr_weekends_sat.get();
		let bhvr_weekends_yes = self_.bhvr_weekends_yes.get();
		let bhvr_wkstart_mon = self_.bhvr_wkstart_mon.get();
		let bhvr_wkstart_sun = self_.bhvr_wkstart_sun.get();
		settings.connect_changed(
			None,
			glib::clone!(
				@weak bhvr_weekends_no,
				@weak bhvr_weekends_sat,
				@weak bhvr_weekends_yes,
				@weak bhvr_wkstart_mon,
				@weak bhvr_wkstart_sun
				=> move |settings, key| {
				match key {
					"bhvr-weekends" => update_weekends(settings, &bhvr_weekends_no, &bhvr_weekends_sat, &bhvr_weekends_yes),
					"bhvr-weekstart" => update_weekstart(settings, &bhvr_wkstart_mon, &bhvr_wkstart_sun),
					_ => ()
				}
			})
		);

		fn btn_update_enum(btn: &ToggleButton, settings: &Settings, key: &'static str, value: i32) {
			if btn.is_active() {
				settings.set_enum(key, value).unwrap()
			}
		}

		update_weekends(&settings, &bhvr_weekends_no, &bhvr_weekends_sat, &bhvr_weekends_yes);
		update_weekstart(&settings, &bhvr_wkstart_mon, &bhvr_wkstart_sun);
		self_
			.bhvr_weekends_yes
			.bind_property("active", &self_.bhvr_wkstart.get(), "sensitive")
			.flags(glib::BindingFlags::SYNC_CREATE)
			.build();

		bhvr_weekends_no.connect_toggled(
			glib::clone!(@weak settings => move |btn| btn_update_enum(btn, &settings, "bhvr-weekends", 0))
		);
		bhvr_weekends_sat.connect_toggled(
			glib::clone!(@weak settings => move |btn| btn_update_enum(btn, &settings, "bhvr-weekends", 1))
		);
		bhvr_weekends_yes.connect_toggled(
			glib::clone!(@weak settings => move |btn| btn_update_enum(btn, &settings, "bhvr-weekends", 2))
		);
		bhvr_wkstart_mon.connect_toggled(
			glib::clone!(@weak settings => move |btn| btn_update_enum(btn, &settings, "bhvr-weekstart", 0))
		);
		bhvr_wkstart_sun.connect_toggled(
			glib::clone!(@weak settings => move |btn| btn_update_enum(btn, &settings, "bhvr-weekstart", 1))
		);

		// BEGIN Appearance
		settings
			.bind("tt-blocksize", &self_.tt_blocksize.get(), "value")
			.flags(gio::SettingsBindFlags::DEFAULT)
			.build();

		// BEGIN Online Accounts
		match Secrets::ensure_unlocked() {
			Ok(_) => {
				// BEGIN Microsoft Office
				settings
					.bind("ms-username", &self_.o365_user.get(), "text")
					.flags(gio::SettingsBindFlags::GET)
					.build();

				match Secrets::get_secret(ServiceProvider::MicrosoftO365, &settings.string("ms-username")) {
					Ok(passwd) => {
						if let Some(passwd) = passwd {
							self_.o365_pass.set_text(&passwd);
							self_.o365_delete.set_sensitive(true);
						}
					},
					Err(err) => {
						self_.o365_pass.set_sensitive(false);
						self_.o365_btn.set_sensitive(false);
						eprintln!("Error getting password from secret service: {}", err);
					}
				}

				{
					let btn = self_.o365_btn.get();
					let delete_button = self_.o365_delete.get();

					let user = self_.o365_user.get();
					user.connect_changed(glib::clone!(@weak btn => move |_| {
						btn.remove_css_class("success-action");
						btn.remove_css_class("fail-action");
					}));

					let pass = self_.o365_pass.get();
					pass.connect_changed(glib::clone!(@weak btn => move |_| {
						btn.remove_css_class("success-action");
						btn.remove_css_class("fail-action");
					}));

					btn.connect_clicked(
						glib::clone!(@weak settings, @weak user, @weak pass, @weak delete_button => move |btn| {
							let account = user.text();
							let passwd = pass.text();
							let user = sm::SmOfficeUser {
								email: account.to_string(),
								password: passwd.to_string()
							};

							btn.set_sensitive(false);
							ctx.spawn_local(glib::clone!(@weak btn => async move {
								match Schulmanager::login_office(user).await {
									Ok(_) => {
										settings.set_string("ms-username", &account).unwrap();
										Secrets::set_secret(ServiceProvider::MicrosoftO365, &account, &passwd).unwrap();

										btn.add_css_class("success-action");
										delete_button.set_sensitive(true);
									},
									Err(err) => {
										btn.add_css_class("fail-action");
										eprintln!("{}", err);
									}
								};
								btn.set_sensitive(true);
							}));
						})
					);

					delete_button.connect_clicked(
						glib::clone!(@weak self as window, @weak settings, @weak pass => move |btn| {
							let diag = MessageDialog::builder()
								.message_type(MessageType::Question)
								.text("Confirm removal")
								.secondary_text("Are you sure you want to remove your Microsoft O365 account?")
								.buttons(ButtonsType::YesNo)
								.build();
							diag.set_transient_for(Some(&window));
							diag.set_modal(true);

							diag.connect_response(glib::clone!(@weak btn => move |diag, resp| {
								match resp {
									gtk::ResponseType::Yes => {
										let account = settings.string("ms-username");
										let _ = Secrets::remove_secret(ServiceProvider::MicrosoftO365, &account);
										let _ = Secrets::remove_secret(ServiceProvider::SchulmanagerJwt, &account);
										settings.reset("ms-username");
										pass.set_text("");

										btn.set_sensitive(false);
										diag.destroy()
									},
									_ => diag.destroy()
								}
							}));

							diag.show();
						})
					);
				}
			},
			Err(err) => {
				let diag = MessageDialog::builder()
					.message_type(MessageType::Warning)
					.text("Secret Service failure")
					.secondary_text(&err.to_string())
					.buttons(ButtonsType::Close)
					.build();
				diag.set_transient_for(Some(self));
				diag.set_modal(true);
				diag.connect_response(|diag, _resp| diag.destroy());
				self_.online_accounts.set_sensitive(false);
			}
		}

		self_.settings.set(settings).unwrap();
	}
}
