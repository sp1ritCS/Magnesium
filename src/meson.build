conf = configuration_data()
conf.set_quoted('APP_ID', app_id)
conf.set_quoted('PKGDATADIR', efg_pkgdatadir)

configure_file(
    input: 'config.rs.in',
    output: 'config.rs',
    configuration: conf
)

run_command(
  'cp',
  join_paths(meson.current_build_dir(), 'config.rs'),
  join_paths(meson.current_source_dir(), 'config.rs'),
  check: true
)

rust_sources = files(
	'application.rs',
	'config.rs',
	'efg/mod.rs',
	'efg/preferences.rs',
	'efg/timetable/blocks.rs',
	'efg/timetable/mod.rs',
	'main.rs',
	'models/mod.rs',
	'models/secret.rs'
)

sources = [
	cargo_sources,
	rust_sources
]

cargo_script = find_program(join_paths(meson.project_source_root(), 'build-aux/cargo.sh'))
cargo_release = custom_target(
	'cargo-build',
	build_by_default: true,
	input: sources,
	output: 'magnesium',
	console: true,
	install: true,
	install_dir: efg_bindir,
	command: [
		cargo_script,
		meson.current_build_dir(),
		meson.project_source_root(),
		'@OUTPUT@',
		get_option('buildtype'),
		app_bin,
	]
)
