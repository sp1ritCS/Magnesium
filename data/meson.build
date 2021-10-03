styles = []

scss_files = files([])

sassc = find_program('sassc', required: false)
if sassc.found()
	sassc_opts = [ '-a', '-M', '-t', 'expanded' ]
	styles += custom_target('efg_styles',
		input: 'efg.scss',
		output: 'efg.css',
		command: [
		  sassc, sassc_opts, '@INPUT@', '@OUTPUT@',
		],
		depend_files: scss_files,
		build_by_default: true)
	install_data('efg.scss', install_dir : efg_pkgdatadir)
endif

gtkresources = gnome.compile_resources('efg', 'efg.gresource.xml',
  dependencies: styles,
  gresource_bundle: true,
  install: true,
  install_dir: efg_pkgdatadir
)

desktop_conf = configuration_data()
desktop_conf.set('icon', app_id)
configure_file(
    input: '@0@.desktop.in'.format(app_id),
    output: '@0@.desktop'.format(app_id),
    configuration: desktop_conf,
    install: true,
    install_dir: join_paths(efg_datadir, 'applications')
)

install_data('@0@.svg'.format(app_id), install_dir : get_option('datadir')/'icons/hicolor/scalable/apps')
install_data('@0@.png'.format(app_id), install_dir : get_option('datadir')/'icons/hicolor/128x128/apps')

install_data('@0@-symbolic.svg'.format(app_id), install_dir : get_option('datadir')/'icons/hicolor/symbolic/apps')

gnome.compile_schemas(build_by_default: true, depend_files: files('@0@.gschema.xml'.format(app_id)))
install_data('@0@.gschema.xml'.format(app_id), install_dir : get_option('datadir')/'glib-2.0/schemas')

devenv.set('EFG_DEVEL_DATADIR', meson.current_build_dir())
devenv.set('GSETTINGS_SCHEMA_DIR', meson.current_build_dir())
