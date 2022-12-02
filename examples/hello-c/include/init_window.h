#ifndef INIT_WINDOW_INCLUDED
#define INIT_WINDOW_INCLUDED 1

#include <wayland-client.h>
#include <xkbcommon/xkbcommon.h>

#include <stdbool.h>
#include <stdint.h>

#include <EGL/egl.h>
#include <EGL/eglplatform.h>

#include <hello.h>

#define LOG(...) fprintf(stderr, __VA_ARGS__)
#define LOG_ERRNO(...)                                                         \
  fprintf(stderr, "Error : %s\n", strerror(errno));                            \
  fprintf(stderr, __VA_ARGS__)

struct window_context {
  /// Native System informations
  EGLNativeDisplayType native_display;
  EGLNativeWindowType native_window;
  uint16_t window_width, window_height;
  /// EGL display
  EGLDisplay egl_display;
  /// EGL context
  EGLContext egl_context;
  /// EGL surface
  EGLSurface egl_surface;

  /// Wayland display
  struct wl_display *wl_display;

  struct xdg_surface *xdg_surface;
  struct xdg_toplevel *xdg_top_level;
  struct wl_surface *wl_surface;
  struct wl_egl_window *wl_egl_window;
  struct wl_region *wl_region;

  struct Engine *engine;
};

struct pointer {
  struct wl_pointer *wlpointer;
  float x;
  float y;
  uint32_t last_click_button;
  uint32_t last_click_time;
  float last_click_x;
  float last_click_y;

  uint32_t button;
  //   NSTimeInterval	   last_timestamp;
  enum wl_pointer_button_state button_state;

  uint32_t axis_source;

  uint32_t serial;
  struct wui_view *focus;
  struct wui_view *captured;
};
struct cursor {
  struct wl_cursor *cursor;
  struct wl_surface *surface;
  struct wl_cursor_image *image;
  struct wl_buffer *buffer;
};

struct output {
  struct wayland_config *wlconfig;
  struct wl_output *output;
  uint32_t server_output_id;
  struct wl_list link;
  int alloc_x;
  int alloc_y;
  int width;
  int height;
  int transform;
  int scale;
  char *make;
  char *model;

  void *user_data;
};

struct wayland_client {
  struct Engine *engine;
  struct wl_display *wl_display;
  struct wl_registry *wl_registry;
  struct wl_compositor *wl_compositor;
  struct wl_subcompositor *wl_subcompositor;
  struct wl_seat *wl_seat;
  struct wl_keyboard *wl_keyboard;
  struct xdg_wm_base *xdg_wm_base;

  struct wl_data_device_manager *data_device_manager;
  struct zxdg_decoration_manager_v1 *decoration_manager;

  int seat_version;

  struct wl_list output_list;
  int output_count;
  struct wl_list window_list;
  int window_count;

  // last event serial from pointer or keyboard
  uint32_t event_serial;

  // cursor
  struct wl_cursor_theme *cursor_theme;
  struct cursor *cursor;
  struct wl_surface *cursor_surface;

  // pointer
  struct pointer pointer;
  // keyboard
  struct xkb_context *xkb_context;

  struct {
    struct xkb_keymap *keymap;
    struct xkb_state *state;
    xkb_mod_mask_t control_mask;
    xkb_mod_mask_t alt_mask;
    xkb_mod_mask_t shift_mask;
  } xkb;

  int modifiers;
};

#define TRUE 1
#define FALSE 0

#define WINDOW_WIDTH 1280
#define WINDOW_HEIGHT 720

struct wayland_client *create_wayland_client();
void destroy_window(struct window_context *window);
EGLBoolean create_egl_context_for_window(struct window_context *window);
struct window_context *create_window_with_egl_context(struct wayland_client *wl,
                                                      char *title, int width,
                                                      int height);
void window_swap_buffers(struct window_context *window);

extern const struct wl_seat_listener seat_listener;
extern const struct wl_pointer_listener pointer_listener;
// extern const struct wl_keyboard_listener keyboard_listener;
#endif