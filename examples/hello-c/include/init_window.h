#ifndef INIT_WINDOW_INCLUDED
#define INIT_WINDOW_INCLUDED 1

#include <stdbool.h>
#include <stdint.h>

#include <EGL/egl.h>
#include <EGL/eglplatform.h>

#define LOG(...) fprintf(stderr, __VA_ARGS__)
#define LOG_ERRNO(...)                                                         \
  fprintf(stderr, "Error : %s\n", strerror(errno));                            \
  fprintf(stderr, __VA_ARGS__)

struct WindowContext {
  /// Native System informations
  EGLNativeDisplayType native_display;
  EGLNativeWindowType native_window;
  uint16_t window_width, window_height;
  /// EGL display
  EGLDisplay display;
  /// EGL context
  EGLContext context;
  /// EGL surface
  EGLSurface surface;
};

#define TRUE 1
#define FALSE 0

#define WINDOW_WIDTH 1280
#define WINDOW_HEIGHT 720

void setup_wayland();
void destroy_window();
void create_native_window(char *title, int width, int height);
EGLBoolean create_egl_context();
EGLBoolean create_window_with_egl_context(char *title, int width, int height);
void swap_buffers();

#endif