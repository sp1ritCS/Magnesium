project('magnesium',
	version: '0.1.0',
	meson_version: '>= 0.59.0',
	default_options: [ 'warning_level=2'],
)

dependency('gtk4', version: '>= 4.0.0')
dependency('libadwaita-1')

app_id = 'dev.sp1rit.efg.Desktop'
app_bin = 'efg'
efg_prefix = get_option('prefix')
efg_bindir = join_paths(efg_prefix, get_option('bindir'))
efg_localedir = join_paths(efg_prefix, get_option('localedir'))
efg_datadir = join_paths(efg_prefix, get_option('datadir'))
efg_pkgdatadir = join_paths(efg_datadir, meson.project_name())

gnome = import('gnome')
devenv = environment()

cargo_sources = files(
	'Cargo.toml',
	'Cargo.lock',
)

subdir('src')
subdir('data')

# Postinstall script (Update icon cache, gsettings, ...)
gnome.post_install(glib_compile_schemas: true, gtk_update_icon_cache: true, update_desktop_database: true)

meson.add_devenv(devenv)
