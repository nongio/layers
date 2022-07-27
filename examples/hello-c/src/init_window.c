// gcc -o test init_window.c -I. -lwayland-client -lwayland-server
// -lwayland-client-protocol -lwayland-egl -lEGL -lGLESv2
#include <wayland-client-core.h>
#include <wayland-client-protocol.h>
#include <wayland-client.h>
#include <wayland-egl.h> // Wayland EGL MUST be included before EGL headers

#include "init_window.h"
#include <GLES2/gl2.h>
#include <math.h>
#include <stdbool.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <sys/time.h>

#include "init_window.h"
#include "xdg-shell-protocol.h"

struct wl_compositor *compositor = NULL;
struct wl_surface *surface;
struct wl_egl_window *egl_window;
struct wl_region *region;

struct xdg_wm_base *xdg_wm_base;
struct xdg_surface *xdg_surface;
struct xdg_toplevel *xdg_top_level;
struct WindowContext window_context = {.native_display = NULL,
                                       .window_width = 0,
                                       .window_height = 0,
                                       .native_window = 0,
                                       .display = NULL,
                                       .context = NULL,
                                       .surface = NULL};
bool program_alive;
int32_t old_w, old_h;

static void xdg_toplevel_handle_configure(void *data,
                                          struct xdg_toplevel *xdg_toplevel,
                                          int32_t w, int32_t h,
                                          struct wl_array *states) {

  // no window geometry event, ignore
  if (w == 0 && h == 0)
    return;

  // window resized
  if (old_w != w && old_h != h) {
    old_w = w;
    old_h = h;

    wl_egl_window_resize(window_context.native_window, w, h, 0, 0);
    wl_surface_commit(surface);
  }
}

static void xdg_toplevel_handle_close(void *data,
                                      struct xdg_toplevel *xdg_toplevel) {
  // window closed, be sure that this event gets processed
  program_alive = false;
}

struct xdg_toplevel_listener xdg_toplevel_listener = {
    .configure = xdg_toplevel_handle_configure,
    .close = xdg_toplevel_handle_close,
};

static void xdg_surface_configure(void *data, struct xdg_surface *xdg_surface,
                                  uint32_t serial) {
  // confirm that you exist to the compositor
  xdg_surface_ack_configure(xdg_surface, serial);
}

const struct xdg_surface_listener xdg_surface_listener = {
    .configure = xdg_surface_configure,
};

static void xdg_wm_base_ping(void *data, struct xdg_wm_base *xdg_wm_base,
                             uint32_t serial) {
  xdg_wm_base_pong(xdg_wm_base, serial);
}

const struct xdg_wm_base_listener xdg_wm_base_listener = {
    .ping = xdg_wm_base_ping,
};

void create_native_window(char *title, int width, int height) {
  old_w = WINDOW_WIDTH;
  old_h = WINDOW_HEIGHT;

  region = wl_compositor_create_region(compositor);

  wl_region_add(region, 0, 0, width, height);
  wl_surface_set_opaque_region(surface, region);

  struct wl_egl_window *egl_window =
      wl_egl_window_create(surface, width, height);

  if (egl_window == EGL_NO_SURFACE) {
    LOG("No window !?\n");
    exit(1);
  } else
    LOG("Window created !\n");
  window_context.window_width = width;
  window_context.window_height = height;
  window_context.native_window = egl_window;
}

EGLBoolean create_egl_context() {
  EGLint numConfigs;
  EGLint majorVersion;
  EGLint minorVersion;
  EGLContext context;
  EGLSurface surface;
  EGLConfig config;
  EGLint fbAttribs[] = {EGL_SURFACE_TYPE,
                        EGL_WINDOW_BIT,
                        EGL_RENDERABLE_TYPE,
                        EGL_OPENGL_ES2_BIT,
                        EGL_RED_SIZE,
                        8,
                        EGL_GREEN_SIZE,
                        8,
                        EGL_BLUE_SIZE,
                        8,
                        EGL_NONE};
  EGLint contextAttribs[] = {EGL_CONTEXT_CLIENT_VERSION, 2, EGL_NONE, EGL_NONE};
  EGLDisplay display = eglGetDisplay(window_context.native_display);
  if (display == EGL_NO_DISPLAY) {
    LOG("No EGL Display...\n");
    return EGL_FALSE;
  }

  // Initialize EGL
  if (!eglInitialize(display, &majorVersion, &minorVersion)) {
    LOG("No Initialisation...\n");
    return EGL_FALSE;
  }

  // Get configs
  if ((eglGetConfigs(display, NULL, 0, &numConfigs) != EGL_TRUE) ||
      (numConfigs == 0)) {
    LOG("No configuration...\n");
    return EGL_FALSE;
  }

  // Choose config
  if ((eglChooseConfig(display, fbAttribs, &config, 1, &numConfigs) !=
       EGL_TRUE) ||
      (numConfigs != 1)) {
    LOG("No configuration...\n");
    return EGL_FALSE;
  }

  // Create a surface
  surface = eglCreateWindowSurface(display, config,
                                   window_context.native_window, NULL);
  if (surface == EGL_NO_SURFACE) {
    LOG("No surface...\n");
    return EGL_FALSE;
  }

  // Create a GL context
  context = eglCreateContext(display, config, EGL_NO_CONTEXT, contextAttribs);
  if (context == EGL_NO_CONTEXT) {
    LOG("No context...\n");
    return EGL_FALSE;
  }

  // Make the context current
  if (!eglMakeCurrent(display, surface, surface, context)) {
    LOG("Could not make the current window current !\n");
    return EGL_FALSE;
  }

  window_context.display = display;
  window_context.surface = surface;
  window_context.context = context;
  return EGL_TRUE;
}

EGLBoolean create_window_with_egl_context(char *title, int width, int height) {
  surface = wl_compositor_create_surface(compositor);
  xdg_surface = xdg_wm_base_get_xdg_surface(xdg_wm_base, surface);

  xdg_surface_add_listener(xdg_surface, &xdg_surface_listener, NULL);

  xdg_top_level = xdg_surface_get_toplevel(xdg_surface);
  xdg_toplevel_set_title(xdg_top_level, "Wayland EGL example");
  xdg_toplevel_add_listener(xdg_top_level, &xdg_toplevel_listener, NULL);

  wl_surface_commit(surface);

  create_native_window(title, width, height);
  return create_egl_context();
}

void draw() {
  glClearColor(0.5, 0.3, 0.0, 1.0);

  glClear(GL_COLOR_BUFFER_BIT);
}

unsigned long last_click = 0;
void swap_buffers() {
  eglSwapBuffers(window_context.display, window_context.surface);
}

static void global_registry_handler(void *data, struct wl_registry *registry,
                                    uint32_t id, const char *interface,
                                    uint32_t version) {
  LOG("Got a registry event for %s id %d\n", interface, id);
  if (strcmp(interface, "wl_compositor") == 0)
    compositor = wl_registry_bind(registry, id, &wl_compositor_interface, 1);
  else if (strcmp(interface, xdg_wm_base_interface.name) == 0) {
    xdg_wm_base = wl_registry_bind(registry, id, &xdg_wm_base_interface, 1);
    xdg_wm_base_add_listener(xdg_wm_base, &xdg_wm_base_listener, NULL);
  }
}

static void global_registry_remover(void *data, struct wl_registry *registry,
                                    uint32_t id) {
  LOG("Got a registry losing event for %d\n", id);
}

const struct wl_registry_listener listener = {global_registry_handler,
                                              global_registry_remover};

void setup_wayland() {

  struct wl_display *display = wl_display_connect(NULL);
  if (display == NULL) {
    LOG("Can't connect to wayland display !?\n");
    exit(1);
  }

  struct wl_registry *wl_registry = wl_display_get_registry(display);
  wl_registry_add_listener(wl_registry, &listener, NULL);

  // This call the attached listener global_registry_handler
  wl_display_dispatch(display);
  wl_display_roundtrip(display);

  if (compositor == NULL || xdg_wm_base == NULL) {
    LOG("No compositor !? No XDG !! There's NOTHING in here !\n");
    exit(1);
  } else {
    LOG("Okay, we got a compositor and a shell... That's something !\n");
    window_context.native_display = display;
  }
}

void destroy_window() {
  eglDestroySurface(window_context.display, window_context.surface);
  wl_egl_window_destroy(window_context.native_window);
  xdg_toplevel_destroy(xdg_top_level);
  xdg_surface_destroy(xdg_surface);
  wl_surface_destroy(surface);
  eglDestroyContext(window_context.display, window_context.context);
}