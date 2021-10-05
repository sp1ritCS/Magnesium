use std::collections::BTreeMap;

use gtk::{gio, glib, prelude::*, subclass::prelude::*, CompositeTemplate};
use chrono::{Date, Local, TimeZone, Weekday};
use sm::transformers::smartv2 as smart;

mod imp {
	use gtk::Label;

	use super::*;

	#[derive(Debug, Default, CompositeTemplate)]
	#[template(resource = "/dev/sp1rit/efg/Desktop/ui/block.ui")]
	pub struct EfgTimetableBlock {
		#[template_child]
		pub subject: TemplateChild<Label>,
		#[template_child]
		pub teacher: TemplateChild<Label>,
		#[template_child]
		pub room: TemplateChild<Label>,
		#[template_child]
		pub time: TemplateChild<Label>,

		#[template_child]
		pub subst: TemplateChild<gtk::MenuButton>,
		#[template_child]
		pub subst_subject: TemplateChild<Label>,
		#[template_child]
		pub subst_teacher: TemplateChild<Label>,
		#[template_child]
		pub subst_room: TemplateChild<Label>
	}

	#[glib::object_subclass]
	impl ObjectSubclass for EfgTimetableBlock {
		type ParentType = gtk::Frame;
		type Type = super::EfgTimetableBlock;

		const NAME: &'static str = "EfgTimetableBlock";

		fn class_init(klass: &mut Self::Class) {
			Self::bind_template(klass);
		}

		fn instance_init(obj: &glib::subclass::InitializingObject<Self>) {
			obj.init_template();
		}
	}

	impl ObjectImpl for EfgTimetableBlock {
		fn constructed(&self, obj: &Self::Type) {
			obj.add_css_class("timetableblock");
			self.parent_constructed(obj);
		}
	}

	impl WidgetImpl for EfgTimetableBlock {}
	impl FrameImpl for EfgTimetableBlock {}
}

glib::wrapper! {
	pub struct EfgTimetableBlock(ObjectSubclass<imp::EfgTimetableBlock>)
		@extends gtk::Widget, gtk::Frame,
		@implements gio::ActionMap, gio::ActionGroup;
}

impl EfgTimetableBlock {
	pub fn new() -> Self {
		glib::Object::new(&[]).expect("Failed to create EfgTimetableBlock")
	}

	fn _format_teachers(&self, teachers: &Vec<smart::Teacher>) -> String {
		let mut res = String::new();
		for (i, teacher) in teachers.iter().enumerate() {
			if let Some(lastname) = &teacher.lastname {
				res.push_str(&lastname);
				if let Some(firstname) = &teacher.firstname {
					res.push_str(&format!(", {}", firstname));
				}
			} else {
				res.push_str(&teacher.abbreviation);
			}

			if i < teachers.len() - 1 {
				res.push_str("; ");
			}
		}

		res
	}

	fn _fill(&self, lesson: &smart::Lesson) {
		let self_ = imp::EfgTimetableBlock::from_instance(self);

		self_.subject.set_label(&lesson.subject.name);
		self_.teacher.set_label(&self._format_teachers(&lesson.teachers));
		self_.room.set_label(&lesson.room);
	}

	fn initialize(&self, element: &smart::TimetableElement) {
		let self_ = imp::EfgTimetableBlock::from_instance(self);

		match element {
			smart::TimetableElement::Lesson(lesson) => self._fill(lesson),
			smart::TimetableElement::Substitution(subst, lesson) => {
				self._fill(subst);

				self_.subst_subject.set_label(&lesson.subject.name);
				self_.subst_teacher.set_label(&self._format_teachers(&lesson.teachers));
				self_.subst_room.set_label(&lesson.room);
				self_.subst.set_visible(true);
			},
			smart::TimetableElement::Cancelled(lesson) => {
				self._fill(lesson);
				self.add_css_class("cancelled");
			},
			smart::TimetableElement::Event(event) => {
				self_.subject.set_label(&event.text);
				self.add_css_class("event");
			}
		}
	}

	fn set_hour_label(&self, text: &str) {
		let self_ = imp::EfgTimetableBlock::from_instance(self);
		self_.time.set_label(text);
	}
}

pub fn get_blocks(
	settings: &gio::Settings,
	hours: &sm::SchoolHoursMap,
	elements: &BTreeMap<usize, Vec<smart::TimetableElement>>,
	day: (usize, &str),
	active: bool,
	max: usize,
	sizegrp: &gtk::SizeGroup
) -> adw::Clamp {
	let blocks = gtk::Box::new(gtk::Orientation::Vertical, 16);
	blocks.set_margin_top(16);
	blocks.set_margin_bottom(16);

	let dayw = gtk::Label::new(Some(day.1));
	blocks.append(&dayw);

	for i in 1..=max {
		let widget: gtk::Widget = elements
			.get(&i)
			.map(|element| {
				let view = gtk::ScrolledWindow::builder()
				.hscrollbar_policy(gtk::PolicyType::Automatic)
				.vscrollbar_policy(gtk::PolicyType::Never)
				.propagate_natural_width(false)
				//.min_content_width(256)
				.build();

				settings
					.bind("tt-blocksize", &view, "width_request")
					.flags(gio::SettingsBindFlags::GET)
					.build();

				/*
				 * this GtkScrolledWindow eats the horizontal scroll events for the
				 * entire day scrolled window and gtk doesn't seem to provide a
				 * proper method to prevent it
				 */
				let viewctl = view.observe_controllers();
				for i in 0..viewctl.n_items() {
					// TODO: this breaks shift-scrolling the inner container
					if let Some(scrollctl) = viewctl
						.item(i)
						.map(|item| item.downcast::<gtk::EventControllerScroll>().ok())
						.flatten()
					{
						let mut flags = gtk::EventControllerScrollFlags::empty();
						flags.insert(gtk::EventControllerScrollFlags::HORIZONTAL);
						flags.insert(gtk::EventControllerScrollFlags::KINETIC);
						scrollctl.set_flags(flags);
					}
				}

				let lbck = gtk::Box::new(gtk::Orientation::Horizontal, 0);

				let hour = hours.get(&i).map(|hs| hs[day.0 - 1]);
				for lesson in element {
					let efg = EfgTimetableBlock::new();
					efg.initialize(lesson);
					if let Some((begin, end)) = hour {
						let text = format!("{} - {}", begin.format("%H:%M"), end.format("%H:%M"));
						efg.set_hour_label(&text);
					}
					lbck.append(&efg);
				}

				view.set_child(Some(&lbck));

				view.upcast()
			})
			.unwrap_or(adw::Bin::new().upcast());

		sizegrp.add_widget(&widget);
		blocks.append(&widget);
	}

	let clamp = adw::Clamp::builder()
		.maximum_size(512)
		.tightening_threshold(256)
		.orientation(gtk::Orientation::Horizontal)
		.hexpand(true)
		.child(&blocks)
		.build();

	clamp.add_css_class("daycol");
	if active {
		clamp.add_css_class("active");
	}

	clamp
}

pub fn get_week(
	settings: &gio::Settings,
	hours: &sm::SmHours,
	table: &smart::DayMap,
	active: glib::DateTime,
	calendar: &gtk::Calendar
) -> adw::Leaflet {
	let sizeg = gtk::SizeGroup::new(gtk::SizeGroupMode::Vertical);

	let body = adw::Leaflet::builder()
		.halign(gtk::Align::Fill)
		.hexpand(true)
		.can_swipe_back(false)
		.can_swipe_forward(false)
		.transition_type(adw::LeafletTransitionType::Slide)
		.build();

	let days = match settings.enum_("bhvr-weekends") {
		0 => 5,
		1 => 6,
		2 => 7,
		_ => 5
	};

	let hours = hours.parse().unwrap();
	let long_day = table
		.map
		.values()
		.map(|day| day.keys().map(|len| *len).max().unwrap_or_default())
		.max()
		.unwrap_or_default();

	for i in 1..=days {
		let (daystr, chrono_day) = match i {
			1 => ("Monday", Weekday::Mon),
			2 => ("Tuesday", Weekday::Tue),
			3 => ("Wednesday", Weekday::Wed),
			4 => ("Thursday", Weekday::Thu),
			5 => ("Friday", Weekday::Fri),
			6 => ("Saturday", Weekday::Sat),
			7 => ("Sunday", Weekday::Sun),
			_ => unreachable!("creating more than seven days")
		};

		let date: Date<Local> = Local.isoywd(active.year(), active.week_of_year() as u32, chrono_day);
		let fallback = BTreeMap::new();
		let day = table.map.get(&date.naive_local()).unwrap_or(&fallback);

		let now = glib::DateTime::new_now_local().unwrap();
		let blocks = get_blocks(
			settings,
			&hours,
			day,
			(i as usize, daystr),
			i == now.day_of_week() && active.week_of_year() == now.week_of_year() && active.year() == now.year(),
			long_day,
			&sizeg
		);
		body.append(&blocks);

		if i == active.day_of_week() {
			body.set_visible_child(&blocks);
		}
	}

	let gesture = gtk::GestureSwipe::new();
	gesture.connect_swipe(glib::clone!(@weak calendar => move |_, x, y| {
		/*
		 * Gesture direction validation
		 * 1. checks if the y velocity is not larger than 512 in both directions
		 * 2. match x velocity (cast to an integer) against:
		 *    1. 0 if x velocity isn't bigger than 1024 in either direction
		 *    2. test direction of velocity and return correct diff. value
		 *    3. catch-all that returns 0, if x is neither positive nor negative (0)
		 *
		 */
		if (-512.0..=512.0).contains(&y) {
			let addr = match x as i64 {
				-1024..=1024 => 0,
				x if x.is_positive() => -1,
				x if x.is_negative() => 1,
				_ => 0
			};

			let new = calendar.date().add_days(addr).unwrap();
			calendar.select_day(&new);
		}
	}));
	body.add_controller(&gesture);

	body
}
