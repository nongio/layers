#include <EAS.hpp>
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

#include "init_window.h"
//
// main is where all program execution starts
//

extern bool program_alive;
extern struct WindowContext window_context;

int main(int argc, char **argv) {
  // using namespace eas;

  printf("Hello there.\n");
  setup_wayland();

  create_window_with_egl_context("Nya", 1280, 720);

  program_alive = true;

  while (program_alive) {
    wl_display_dispatch_pending(window_context.native_display);
    draw();
    swap_buffers();
  }

  destroy_window();
  wl_display_disconnect(window_context.native_display);
  LOG("Display disconnected !\n");

  Scene *scene = new_scene();

  // Layer *layer = new_layer();
  // Layer *child_layer = new_layer();

  // // append a layer to the current scene
  // entity_append_layer(scene.root(), layer);
  // entity_append_layer(layer.id(), child_layer);

  // scene.commit(model_set_position(layer, 0, 0, 0));

  // // queue some modelchanges associated with an animation
  // scene.animate(Transition{.duration = 2.0, .delay = 0.0, .easing = {}},
  //               model_set_position(layer, 0, 0, 0));

  // // queue some model changes to be executed as soon as possible
  // scene.commit(model_to(layer, Layer{.position = {0, 0}, .scale =
  // {1.1, 1.1}}));

  // // const ModelChange<Point> *change =
  // // model_change_position(layer, 50.0, 50.0);

  // state_commit(state, change);

  // state_update(state, 0.01);

  return 0;
}
