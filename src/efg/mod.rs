use std::{cell::RefCell, rc::Rc};

use gtk::{gio, glib, prelude::*, subclass::prelude::*, CompositeTemplate};
use adw::ApplicationWindow;
use once_cell::sync::OnceCell;

use crate::models::secret::{Secrets, ServiceProvider};

pub mod preferences;
mod timetable;

#[derive(Debug)]
pub enum SchulmanagerState {
	Available(Rc<sm::Schulmanager>),
	NoCredentials,
	Error(anyhow::Error),
	Uninitialized
}
impl Default for SchulmanagerState {
	fn default() -> Self {
		Self::Uninitialized
	}
}

pub type SchulmanagerStatePtr = Rc<RefCell<SchulmanagerState>>;

pub mod imp {
	use adw::{subclass::prelude::*, ApplicationWindow};

	use super::*;
	#[derive(Debug, Default, CompositeTemplate)]
	#[template(resource = "/dev/sp1rit/efg/Desktop/ui/window.ui")]
	pub struct EfgWindow {
		pub settings: OnceCell<gio::Settings>,

		pub schulmanager: SchulmanagerStatePtr,

		#[template_child]
		pub greeting: TemplateChild<gtk::Label>,
		#[template_child]
		pub timetable: TemplateChild<timetable::EfgTimetable>
	}

	#[glib::object_subclass]
	impl ObjectSubclass for EfgWindow {
		type ParentType = ApplicationWindow;
		type Type = super::EfgWindow;

		const NAME: &'static str = "EfgWindow";

		fn class_init(klass: &mut Self::Class) {
			Self::bind_template(klass);
		}

		fn instance_init(obj: &glib::subclass::InitializingObject<Self>) {
			obj.init_template();
		}
	}

	impl ObjectImpl for EfgWindow {
		fn constructed(&self, obj: &Self::Type) {
			obj.setup_triggers();
			obj.setup_greeting();
			self.settings.set(gio::Settings::new(crate::config::APP_ID)).unwrap();
			self.parent_constructed(obj);
			obj.setup_schulmanager();
		}
	}

	impl WidgetImpl for EfgWindow {}
	impl WindowImpl for EfgWindow {}
	impl AdwWindowImpl for EfgWindow {}
	impl ApplicationWindowImpl for EfgWindow {}
	impl AdwApplicationWindowImpl for EfgWindow {}
}

glib::wrapper! {
	pub struct EfgWindow(ObjectSubclass<imp::EfgWindow>)
		@extends gtk::Widget, gtk::Window, gtk::ApplicationWindow, ApplicationWindow,
		@implements gio::ActionMap, gio::ActionGroup;
}

impl EfgWindow {
	pub fn new<P: glib::IsA<gtk::Application>>(app: &P) -> Self {
		glib::Object::new(&[("application", app)]).expect("Failed to create EfgWindow")
	}

	fn setup_greeting(&self) {
		let self_ = imp::EfgWindow::from_instance(self);

		let username = unsafe {
			let passwd: Option<&libc::passwd> = libc::getpwuid(libc::getuid()).as_ref();
			passwd
				.map(|pw| {
					let ptr = pw.pw_gecos;
					match ptr.is_null() {
						false => std::ffi::CStr::from_ptr(ptr).to_str().ok(),
						true => None
					}
				})
				.flatten()
		};

		if let Some(username) = username.map(|name| name.split(' ').next()).flatten() {
			self_.greeting.set_label(&format!("Hello {},", username));
		}
	}

	fn setup_schulmanager(&self) {
		let self_ = imp::EfgWindow::from_instance(self);
		self_.timetable.register_schulmanager(self_.schulmanager.clone());

		let ctx = glib::MainContext::default();
		let settings = self_.settings.get().unwrap();

		let timetable = self_.timetable.get();
		let gschulmanager = self_.schulmanager.clone();

		match Secrets::ensure_unlocked() {
			Ok(_) => {
				ctx.spawn_local(glib::clone!(@weak settings, @weak timetable => async move {
					let account = settings.string("ms-username");
					let schulmanager = if account != "" {
						async fn do_office_auth(account: glib::GString) -> SchulmanagerState {
							match Secrets::get_secret(ServiceProvider::MicrosoftO365, &account) {
								Ok(password) => match password {
									Some(password) => {
										let user = sm::SmOfficeUser {
											email: account.to_string(),
											password: password
										};
										match sm::Schulmanager::login_office(user).await {
											Ok(schulmanager) => {
												Secrets::set_secret(ServiceProvider::SchulmanagerJwt, &account, &schulmanager.token).unwrap();
												SchulmanagerState::Available(Rc::new(schulmanager))
											},
											Err(err) => SchulmanagerState::Error(err)
										}
									},
									None => SchulmanagerState::NoCredentials
								},
								Err(err) => SchulmanagerState::Error(err.into())
							}
						}

						match Secrets::get_secret(ServiceProvider::SchulmanagerJwt, &account) {
							Ok(token) => match token {
								Some(token) => {
									match sm::Schulmanager::new(sm::ClientAuthMethod::JwtAuth(token)).await {
										Ok(schulmanager) => {
											Secrets::set_secret(ServiceProvider::SchulmanagerJwt, &account, &schulmanager.token).unwrap();
											SchulmanagerState::Available(Rc::new(schulmanager))
										},
										Err(_) => do_office_auth(account).await
									}
								},
								None => do_office_auth(account).await
							},
							Err(err) => SchulmanagerState::Error(err.into())
						}
					} else {
						SchulmanagerState::NoCredentials
					};

					{ *gschulmanager.borrow_mut() = schulmanager; }
					timetable.trigger_schulmanager_update();
				}));
			},
			Err(err) => {
				{
					*gschulmanager.borrow_mut() = SchulmanagerState::Error(err.into());
				}
				timetable.trigger_schulmanager_update();
			}
		}
	}

	pub fn setup_triggers(&self) {
		let display = self.display();

		let theme = gtk::IconTheme::for_display(&display).unwrap();
		theme.add_resource_path("/dev/sp1rit/efg/Desktop/icons/");

		let stylemgr = adw::StyleManager::for_display(&display).unwrap();

		stylemgr.connect_dark_notify(glib::clone!(@weak self as window => move |mgr| {
			if mgr.is_dark() {
				window.add_css_class("gdark");
			} else {
				window.remove_css_class("gdark");
			}
		}));

		//stylemgr.set_color_scheme(adw::ColorScheme::PreferDark);
	}
}
