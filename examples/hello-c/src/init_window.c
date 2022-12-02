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

bool program_alive;
int32_t old_w, old_h;

static void xdg_toplevel_handle_configure(void *data,
                                          struct xdg_toplevel *xdg_toplevel,
                                          int32_t w, int32_t h,
                                          struct wl_array *states) {

  struct window_context *window = data;
  // no window geometry event, ignore
  if (w == 0 && h == 0)
    return;

  // window resized
  if (old_w != w && old_h != h) {
    old_w = w;
    old_h = h;

    wl_egl_window_resize(window->native_window, w, h, 0, 0);
    wl_surface_commit(window->wl_surface);
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

EGLBoolean create_egl_context_for_window(struct window_context *window) {
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
  EGLDisplay display = eglGetDisplay(window->native_display);
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
  surface =
      eglCreateWindowSurface(display, config, window->native_window, NULL);
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

  window->egl_surface = surface;
  window->egl_context = context;
  window->egl_display = display;
  LOG("EGL context created !\n");
  return EGL_TRUE;
}

struct window_context *create_window_with_egl_context(struct wayland_client *wl,
                                                      char *title, int width,
                                                      int height) {
  struct window_context *win = malloc(sizeof(struct window_context));
  win->native_display = wl->wl_display;
  win->wl_display = wl->wl_display;

  win->wl_surface = wl_compositor_create_surface(wl->wl_compositor);
  win->xdg_surface =
      xdg_wm_base_get_xdg_surface(wl->xdg_wm_base, win->wl_surface);

  xdg_surface_add_listener(win->xdg_surface, &xdg_surface_listener, win);

  win->xdg_top_level = xdg_surface_get_toplevel(win->xdg_surface);
  xdg_toplevel_set_title(win->xdg_top_level, title);
  xdg_toplevel_add_listener(win->xdg_top_level, &xdg_toplevel_listener, win);

  wl_surface_commit(win->wl_surface);

  old_w = WINDOW_WIDTH;
  old_h = WINDOW_HEIGHT;

  win->wl_region = wl_compositor_create_region(wl->wl_compositor);

  wl_region_add(win->wl_region, 0, 0, width, height);
  wl_surface_set_opaque_region(win->wl_surface, win->wl_region);

  struct wl_egl_window *egl_window =
      wl_egl_window_create(win->wl_surface, width, height);

  if (egl_window == EGL_NO_SURFACE) {
    LOG("No window !?\n");
    exit(1);
  } else {
    LOG("Window created !\n");
  }
  win->window_width = width;
  win->window_height = height;
  win->native_window = egl_window;

  if (create_egl_context_for_window(win)) {
    return win;
  } else {
    return NULL;
  }
}

unsigned long last_click = 0;
void window_swap_buffers(struct window_context *window) {
  eglSwapBuffers(window->egl_display, window->egl_surface);
}

static void global_registry_handler(void *data, struct wl_registry *registry,
                                    uint32_t id, const char *interface,
                                    uint32_t version) {
  struct wayland_client *wl = (struct wayland_client *)data;

  LOG("Got a registry event for %s id %d\n", interface, id);
  if (strcmp(interface, "wl_compositor") == 0) {
    wl->wl_compositor =
        wl_registry_bind(registry, id, &wl_compositor_interface, 1);
  } else if (strcmp(interface, xdg_wm_base_interface.name) == 0) {
    wl->xdg_wm_base = wl_registry_bind(registry, id, &xdg_wm_base_interface, 1);
    xdg_wm_base_add_listener(wl->xdg_wm_base, &xdg_wm_base_listener, NULL);
  } else if (strcmp(interface, wl_seat_interface.name) == 0) {
    wl->pointer.wlpointer = NULL;
    wl->seat_version = version;
    wl->wl_seat =
        (struct wl_seat *)wl_registry_bind(registry, id, &wl_seat_interface, 1);
    wl_seat_add_listener(wl->wl_seat, &seat_listener, wl);
  }
}

static void global_registry_remover(void *data, struct wl_registry *registry,
                                    uint32_t id) {
  LOG("Got a registry losing event for %d\n", id);
}

const struct wl_registry_listener listener = {global_registry_handler,
                                              global_registry_remover};
struct wayland_client *wl;

struct wayland_client *create_wayland_client() {
  wl = malloc(sizeof(struct wayland_client));
  struct wl_display *wl_display = wl_display_connect(NULL);

  if (wl_display == NULL) {
    LOG("Can't connect to wayland display !?\n");
    exit(1);
  }

  struct wl_registry *wl_registry = wl_display_get_registry(wl_display);
  wl_registry_add_listener(wl_registry, &listener, wl);

  // This call the attached listener global_registry_handler
  wl_display_dispatch(wl_display);
  wl_display_roundtrip(wl_display);

  if (wl->wl_compositor == NULL || wl->xdg_wm_base == NULL) {
    LOG("No compositor !? No XDG !! There's NOTHING in here !\n");
    exit(1);
  }
  wl->wl_display = wl_display;
  wl->wl_registry = wl_registry;

  return wl;
}

void destroy_window(struct window_context *window) {
  eglDestroySurface(window->egl_display, window->egl_surface);
  eglDestroyContext(window->egl_display, window->egl_context);
  wl_egl_window_destroy(window->native_window);
  xdg_toplevel_destroy(window->xdg_top_level);
  xdg_surface_destroy(window->xdg_surface);
  wl_surface_destroy(window->wl_surface);
}