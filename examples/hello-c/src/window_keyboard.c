#include "init_window.h"

static void keyboard_handle_key(void *data, struct wl_keyboard *keyboard,
                                uint32_t serial, uint32_t time, uint32_t key,
                                uint32_t state_w) {
  struct wayland_client *wl = (struct wayland_client *)data;
  wl->event_serial = serial;
}

const struct wl_keyboard_listener keyboard_listener = {
    // .keymap = keyboard_handle_keymap,
    // .enter = keyboard_handle_enter,
    // .leave = keyboard_handle_leave,
    .key = keyboard_handle_key,
    // .modifiers = keyboard_handle_modifiers,
    // .repeat_info = keyboard_handle_repeat_info
};