use std::{cell::RefCell, rc::Rc};

use gtk::{gio, glib, prelude::*, subclass::prelude::*, CompositeTemplate};
use once_cell::unsync::OnceCell;

use crate::efg::{SchulmanagerState, SchulmanagerStatePtr};

mod blocks;

mod imp {
	use super::*;

	#[derive(Debug, Default, CompositeTemplate)]
	#[template(resource = "/dev/sp1rit/efg/Desktop/ui/timetable.ui")]
	pub struct EfgTimetable {
		pub settings: OnceCell<gio::Settings>,
		pub schulmanager: OnceCell<SchulmanagerStatePtr>,

		pub current: Rc<RefCell<Option<gtk::Widget>>>,
		#[template_child]
		pub spinner: TemplateChild<gtk::Spinner>,
		#[template_child]
		pub stack: TemplateChild<gtk::Stack>,
		#[template_child]
		pub view: TemplateChild<gtk::ScrolledWindow>,

		pub active_date: Rc<RefCell<Option<glib::DateTime>>>,
		#[template_child]
		pub calendar: TemplateChild<gtk::Calendar>,
		#[template_child]
		pub date: TemplateChild<gtk::Entry>,

		#[template_child]
		pub lbtn: TemplateChild<gtk::Button>,
		#[template_child]
		pub rbtn: TemplateChild<gtk::Button>,

		#[template_child]
		pub nav: TemplateChild<gtk::Box>
	}

	#[glib::object_subclass]
	impl ObjectSubclass for EfgTimetable {
		type ParentType = gtk::Box;
		type Type = super::EfgTimetable;

		const NAME: &'static str = "EfgTimetable";

		fn class_init(klass: &mut Self::Class) {
			Self::bind_template(klass);
		}

		fn instance_init(obj: &glib::subclass::InitializingObject<Self>) {
			obj.init_template();
		}
	}

	impl ObjectImpl for EfgTimetable {
		fn constructed(&self, obj: &Self::Type) {
			obj.setup_triggers();
			self.parent_constructed(obj);
		}
	}

	impl WidgetImpl for EfgTimetable {}
	impl BoxImpl for EfgTimetable {}
}

glib::wrapper! {
	pub struct EfgTimetable(ObjectSubclass<imp::EfgTimetable>)
		@extends gtk::Widget, gtk::Box,
		@implements gio::ActionMap, gio::ActionGroup;
}

impl EfgTimetable {
	fn setup_triggers(&self) {
		let self_ = imp::EfgTimetable::from_instance(self);
		let settings = gio::Settings::new(crate::config::APP_ID);
		self_.settings.set(settings).unwrap();

		let calendar = self_.calendar.get();
		let current = self_.current.clone();
		self_.lbtn.connect_clicked(glib::clone!(@weak calendar => move |_| {
			let delta = if let Some(leaflet) = current.borrow().as_ref().map(|widget| widget.downcast_ref::<adw::Leaflet>()).flatten() {
				match leaflet.is_folded() {
					true => -1,
					false => -7
				}
			} else { -1 };
			let new = calendar.date().add_days(delta).unwrap();
			calendar.select_day(&new);
		}));
		let current = self_.current.clone();
		self_.rbtn.connect_clicked(glib::clone!(@weak calendar => move |_| {
			let delta = if let Some(leaflet) = current.borrow().as_ref().map(|widget| widget.downcast_ref::<adw::Leaflet>()).flatten() {
				match leaflet.is_folded() {
					true => 1,
					false => 7
				}
			} else { 1 };
			let new = calendar.date().add_days(delta).unwrap();
			calendar.select_day(&new);
		}));
	}

	pub fn setup_sm_trigger(&self) {
		let ctx = glib::MainContext::default();
		let self_ = imp::EfgTimetable::from_instance(self);

		let settings = self_.settings.get().unwrap();

		let date = self_.date.get();
		let active_date_c = self_.active_date.clone();
		let nav = self_.nav.get();
		let view = self_.view.get();
		let stack = self_.stack.get();
		let spinner = self_.spinner.get();
		let current = self_.current.clone();
		let gschulmanager = self_.schulmanager.get().unwrap().clone();
		self_.calendar.connect_day_selected(
			glib::clone!(@strong ctx, @weak settings, @weak nav, @weak date, @weak view, @weak stack, @weak spinner => move |cal| {
				match &*gschulmanager.borrow() {
					SchulmanagerState::Available(schulmanager) => {
						nav.set_sensitive(true);
						let active_date = active_date_c.take().unwrap_or(cal.date());
						let delta = cal.date().difference(&active_date) / glib::ffi::G_TIME_SPAN_DAY;
						let weekend = settings.enum_("bhvr-weekends");
						if weekend < 2 {
							let wday = cal.date().day_of_week();
							if (6+weekend..=7).contains(&wday) {
								/*
								 * during weekends move date to next monday or last friday depending on delta
								 * delta positive:
								 *    8-6(saturday) = 2 --> monday
								 *    8-7(sunday) = 1 --> monday
								 * delta negative:
								 *    5-6(saturday) = -1 --> friday
								 *    5-7(sunday) = -2 --> friday
								 */
								let den = match delta.is_negative() { // while 0 shoudn not occur as weekday 6..=7 will be redirected, 0 will return 8
									true => 5+weekend,
									false => 8
								};
								let nwdate = cal.date().add_days(den-wday).unwrap();
								cal.select_day(&nwdate);
							}
						}

						let cal_date = cal.date().format("%a, %x").unwrap();
						date.buffer().set_text(&cal_date);

						{
							if cal.date().week_of_year() == active_date.week_of_year() &&
								cal.date().year() == active_date.year() &&
								current.borrow().as_ref().map(|widget| widget.is::<adw::Leaflet>()) == Some(true)
							{
								assert!((-7..=7).contains(&delta));
								let leaflet = current.borrow();
								let leaflet = leaflet.as_ref().unwrap().downcast_ref::<adw::Leaflet>().unwrap();
								let direction = match delta.is_positive() { // keep in mind that 0 is not positive.
									true => adw::NavigationDirection::Forward,
									false => adw::NavigationDirection::Back
								};
								for _ in 0..delta.abs() {
									leaflet.navigate(direction);
								}
							} else {
								// TODO: for some reason there are two leaflets added during startup
								if let Some(widget) = current.take() {
									stack.remove(&widget);
								}
								stack.set_visible_child(&spinner);

								let schulmanager = schulmanager.clone();
								let current = current.clone();
								ctx.spawn_local(glib::clone!(@weak settings, @weak cal, @weak view, @weak stack => async move {
									//async_std::task::sleep(std::time::Duration::from_secs(25)).await;

									let hours = schulmanager.get_hours();
									let timetable = schulmanager.get_timetable(cal.date().week_of_year() as u32, Some(cal.date().year()));

									let (hours, timetable) = futures::join!(hours, timetable);

                                    let smart = timetable.expect("failed getting timetable").to_smart_v2_daymap().expect("failed parsing timetable");

									let newc = blocks::get_week(&settings, &hours.unwrap(), &smart[0], cal.date(), &cal);
									newc.connect_child_transition_running_notify(move |_| {
										// set "view" scrollbar back to top
										view.vadjustment().map(|adj| adj.set_value(0.0));
									});
									stack.add_child(&newc);
									stack.set_visible_child(&newc);
									current.replace(Some(newc.upcast()));
								}));
							}
						}
						active_date_c.replace(Some(cal.date()));
					},
					SchulmanagerState::NoCredentials => {
						nav.set_sensitive(false);
						let statuspg = adw::StatusPage::builder()
							.title("M$ O365 authentication not set-up")
							.description("To authenticate with M$ SSO, enter your credentials under Preferences → Online Accounts.")
							.icon_name("dialog-warning-symbolic")
							.build();

						if let Some(widget) = current.take() {
							stack.remove(&widget);
						}

						stack.add_child(&statuspg);
						stack.set_visible_child(&statuspg);
						current.replace(Some(statuspg.upcast()));
					},
					SchulmanagerState::Error(err) => {
						nav.set_sensitive(false);
						let statuspg = adw::StatusPage::builder()
							.title("Authentication error")
							.description(&err.to_string())
							.icon_name("dialog-warning-symbolic")
							.build();

						if let Some(widget) = current.take() {
							stack.remove(&widget);
						}

						stack.add_child(&statuspg);
						stack.set_visible_child(&statuspg);
						current.replace(Some(statuspg.upcast()));
					},
					SchulmanagerState::Uninitialized => {
						nav.set_sensitive(false);
						stack.set_visible_child(&spinner);
					}
				}
			})
		);
	}

	pub fn register_schulmanager(&self, sm: SchulmanagerStatePtr) {
		let self_ = imp::EfgTimetable::from_instance(self);
		self_.schulmanager.set(sm).unwrap();

		self.setup_sm_trigger();
	}

	pub fn trigger_schulmanager_update(&self) {
		let self_ = imp::EfgTimetable::from_instance(self);
		self_.calendar.emit_by_name("day_selected", &[]).unwrap();
	}
}
