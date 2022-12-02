// #include <hello.h>

#include "init_window.h"

static void pointer_handle_button(void *data, struct wl_pointer *pointer,
                                  uint32_t serial, uint32_t time,
                                  uint32_t button, uint32_t state_w) {

  struct wayland_client *wl = (struct wayland_client *)data;
  enum wl_pointer_button_state state = (enum wl_pointer_button_state)state_w;

  //   unsigned int eventFlags = wl->modifiers;
  enum ButtonState button_state = ButtonState_Released;
  switch (state) {
  case WL_POINTER_BUTTON_STATE_PRESSED:
    LOG("wl button pressed\n");
button_state =
    /* code */
    break;
  case WL_POINTER_BUTTON_STATE_RELEASED:
    LOG("wl button released\n");
    break;
  default:
    break;
  }
  engine_handle_pointer_button(wl->engine, ButtonState_Released);
}

// triggered when the cursor is over a surface
static void pointer_handle_motion(void *data, struct wl_pointer *pointer,
                                  uint32_t time, wl_fixed_t sx_w,
                                  wl_fixed_t sy_w) {
  struct wayland_client *wl = (struct wayland_client *)data;

  float sx = wl_fixed_to_double(sx_w);
  float sy = wl_fixed_to_double(sy_w);

  //   float deltaX = sx - wl->pointer.x;
  //   float deltaY = sy - wl->pointer.y;

  wl->pointer.x = sx;
  wl->pointer.y = sy;
}

static void pointer_handle_enter(void *data, struct wl_pointer *pointer,
                                 uint32_t serial, struct wl_surface *surface,
                                 wl_fixed_t sx_w, wl_fixed_t sy_w) {}

static void pointer_handle_leave(void *data, struct wl_pointer *pointer,
                                 uint32_t serial, struct wl_surface *surface) {}

const struct wl_pointer_listener pointer_listener = {
    .enter = pointer_handle_enter,
    .leave = pointer_handle_leave,
    .motion = pointer_handle_motion,
    .button = pointer_handle_button,
    // .axis = pointer_handle_axis,
    // .frame = pointer_handle_frame,
    // .axis_source = pointer_handle_axis_source,
    // .axis_stop = pointer_handle_axis_stop,
    // .axis_discrete = pointer_handle_axis_discrete
};