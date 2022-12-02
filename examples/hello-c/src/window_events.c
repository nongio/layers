#include "init_window.h"

static void seat_handle_capabilities(void *data, struct wl_seat *wl_seat,
                                     uint32_t caps) {
  struct wayland_client *wl = (struct wayland_client *)data;

  if ((caps & WL_SEAT_CAPABILITY_POINTER) && !wl->pointer.wlpointer) {
    wl->pointer.wlpointer = wl_seat_get_pointer(wl_seat);
    wl_pointer_set_user_data(wl->pointer.wlpointer, wl);
    wl_pointer_add_listener(wl->pointer.wlpointer, &pointer_listener, wl);
  } else if (!(caps & WL_SEAT_CAPABILITY_POINTER) && wl->pointer.wlpointer) {
    if (wl->seat_version >= WL_POINTER_RELEASE_SINCE_VERSION)
      wl_pointer_release(wl->pointer.wlpointer);
    else
      wl_pointer_destroy(wl->pointer.wlpointer);
    wl->pointer.wlpointer = NULL;
  }

  // wl_display_dispatch_pending(wl->wl_display);
  // wl_display_flush(wl->wl_display);

  if ((caps & WL_SEAT_CAPABILITY_KEYBOARD) && !wl->wl_keyboard) {
    wl->wl_keyboard = wl_seat_get_keyboard(wl_seat);
    wl_keyboard_set_user_data(wl->wl_keyboard, wl);
    // wl_keyboard_add_listener(wl->wl_keyboard, &keyboard_listener, wl);
  } else if (!(caps & WL_SEAT_CAPABILITY_KEYBOARD) && wl->wl_keyboard) {
    if (wl->seat_version >= WL_KEYBOARD_RELEASE_SINCE_VERSION)
      wl_keyboard_release(wl->wl_keyboard);
    else
      wl_keyboard_destroy(wl->wl_keyboard);
    wl->wl_keyboard = NULL;
  }
}

const struct wl_seat_listener seat_listener = {
    .capabilities = seat_handle_capabilities,
};