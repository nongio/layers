#include <math.h>

#include <stdint.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <sys/time.h>

#include <wayland-client-core.h>
#include <wayland-client-protocol.h>
#include <wayland-client.h>

#include <EGL/egl.h>
#include <EGL/eglplatform.h>
#include <GLES2/gl2.h>

#include <hello.h>

#include "init_window.h"

extern bool program_alive;
extern struct WindowContext window_context;

int main(int argc, char **argv) {

  printf("Hello there.\n");
  program_alive = true;

  setup_wayland();

  const struct Engine *engine = engine_create();

  create_window_with_egl_context("Nya", 1280, 720);
  GLint drawFboId = 0;
  glGetIntegerv(GL_FRAMEBUFFER_BINDING, &drawFboId);

  const struct SkiaRenderer *renderer =
      create_skia_renderer(1280, 720, 1, 8, drawFboId);

  struct ModelLayer *layers[100];
  Easing ease_out = {.x1 = 0.0, .y1 = 0.0, .x2 = 0.0, .y2 = 1.0};
  Transition_Easing timing = {
      .duration = 10.0f,
      .delay = 0.0f,
      .timing = ease_out,
  };
  for (int i = 0; i < 100; i++) {
    const struct ModelLayer *layer = layer_create();
    engine_add_layer(engine, layer);
    layers[i] = (struct ModelLayer *)layer;
  }

  for (int i = 0; i < 100; i++) {
    // struct Point position = {.x = 0.0f, .y = 0.0f};
    layer_backgroundcolor_to(layers[i], 100, 80, 90, 100, timing);
    layer_position_to(layers[i], (rand() % 500) * 1.0f,
                      (rand() % 500) * 1.0f - 250.0, timing);
    layer_border_radius_to(layers[i], (rand() % 50) * 1.0f, timing);
  }

  program_alive = true;

  while (program_alive) {
    wl_display_dispatch_pending(window_context.wl_display);
    engine_update(engine, 0.0333);

    render_scene(renderer, engine);

    swap_buffers();
  }

  destroy_window();
  wl_display_disconnect(window_context.wl_display);
  LOG("Display disconnected !\n");

  return 0;
}
