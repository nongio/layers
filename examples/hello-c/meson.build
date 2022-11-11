
project('example', 'cpp', 'c')

run_target('eas',
  command : 'build_library.sh')


cc = meson.get_compiler('c')
libhello = cc.find_library('hello', dirs: ['/home/riccardo/dev/EAS/target/debug'])
include_libeas = ['/home/riccardo/dev/EAS/target/']

wayland_egl = dependency('wayland-egl')
wayland_protos = dependency('wayland-protocols', version: '>=1.14')
wl_protocol_dir = wayland_protos.get_variable(pkgconfig:'pkgdatadir')

server_protocols = [
  [wl_protocol_dir, 'stable/xdg-shell/xdg-shell.xml'],
  # [wl_protocol_dir, 'unstable/xdg-decoration/xdg-decoration-unstable-v1.xml'],
 # ['../screen-composer/protocols/wlr-layer-shell-unstable-v1.xml'],
  # ['../screen-composer/protocols/sc-layer-unstable-v1.xml'],
]

wayland_scanner = find_program('wayland-scanner')

wl_protos_src = []
wl_protos_headers = []

foreach p : server_protocols
	xml = join_paths(p)
	wl_protos_src += custom_target(
		xml.underscorify() + '_client_c',
		input: xml,
		output: '@BASENAME@-protocol.c',
		command: [wayland_scanner, 'public-code', '@INPUT@', '@OUTPUT@'],
	)
	wl_protos_headers += custom_target(
		xml.underscorify() + '_client_h',
		input: xml,
		output: '@BASENAME@-protocol.h',
		command: [wayland_scanner, 'client-header', '@INPUT@', '@OUTPUT@'],
	)
endforeach

lib_client_protos = static_library(
	'client_protos',
	wl_protos_src + wl_protos_headers,
	dependencies: wayland_egl.partial_dependency(compile_args: true),
)

client_protos = declare_dependency(
	link_with: lib_client_protos,
	sources: wl_protos_headers,
)

executable('demo', ['src/main.c', 'src/init_window.c'], dependencies: [libhello,
dependency('egl'),
dependency('glesv2'),
dependency('fontconfig'),
wayland_egl,
client_protos

], include_directories: ['include/', include_libeas])
