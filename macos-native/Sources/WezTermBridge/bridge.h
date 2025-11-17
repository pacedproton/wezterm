/*
 * WezTerm Swift/Rust Bridge
 *
 * This header defines the C FFI interface between Swift and Rust code.
 * It enables native macOS UI components to interact with the Rust core.
 */

#ifndef WEZTERM_BRIDGE_H
#define WEZTERM_BRIDGE_H

#include <stdint.h>
#include <stdbool.h>
#include <stddef.h>

#ifdef __cplusplus
extern "C" {
#endif

/* Opaque types */
typedef struct WezTermConfig WezTermConfig;
typedef struct WezTermTerminal WezTermTerminal;
typedef struct WezTermColorScheme WezTermColorScheme;

/* String result structure */
typedef struct {
    char* ptr;
    size_t len;
    bool owned;
} WezTermString;

/* Color structure */
typedef struct {
    uint8_t r;
    uint8_t g;
    uint8_t b;
    uint8_t a;
} WezTermColor;

/* ========== Configuration API ========== */

/**
 * Create a new configuration object with defaults
 */
WezTermConfig* wezterm_config_new(void);

/**
 * Free a configuration object
 */
void wezterm_config_free(WezTermConfig* config);

/**
 * Apply configuration to running instance
 */
bool wezterm_config_apply(WezTermConfig* config);

/**
 * Load configuration from file
 */
WezTermConfig* wezterm_config_load_from_file(const char* path);

/**
 * Save configuration to file
 */
bool wezterm_config_save_to_file(WezTermConfig* config, const char* path);

/* Font configuration */
void wezterm_config_set_font_family(WezTermConfig* config, const char* family);
void wezterm_config_set_font_size(WezTermConfig* config, double size);
void wezterm_config_set_line_height(WezTermConfig* config, double height);
void wezterm_config_set_font_weight(WezTermConfig* config, uint16_t weight);

const char* wezterm_config_get_font_family(const WezTermConfig* config);
double wezterm_config_get_font_size(const WezTermConfig* config);
double wezterm_config_get_line_height(const WezTermConfig* config);
uint16_t wezterm_config_get_font_weight(const WezTermConfig* config);

/* Color configuration */
void wezterm_config_set_color_scheme(WezTermConfig* config, const char* name);
void wezterm_config_set_foreground_color(WezTermConfig* config, WezTermColor color);
void wezterm_config_set_background_color(WezTermConfig* config, WezTermColor color);
void wezterm_config_set_cursor_color(WezTermConfig* config, WezTermColor color);

const char* wezterm_config_get_color_scheme(const WezTermConfig* config);
WezTermColor wezterm_config_get_foreground_color(const WezTermConfig* config);
WezTermColor wezterm_config_get_background_color(const WezTermConfig* config);

/* Window configuration */
void wezterm_config_set_window_opacity(WezTermConfig* config, double opacity);
void wezterm_config_set_background_blur(WezTermConfig* config, int32_t blur);
void wezterm_config_set_initial_rows(WezTermConfig* config, uint16_t rows);
void wezterm_config_set_initial_cols(WezTermConfig* config, uint16_t cols);
void wezterm_config_set_scrollback_lines(WezTermConfig* config, size_t lines);

double wezterm_config_get_window_opacity(const WezTermConfig* config);
int32_t wezterm_config_get_background_blur(const WezTermConfig* config);
uint16_t wezterm_config_get_initial_rows(const WezTermConfig* config);
uint16_t wezterm_config_get_initial_cols(const WezTermConfig* config);
size_t wezterm_config_get_scrollback_lines(const WezTermConfig* config);

/* Cursor configuration */
typedef enum {
    WEZTERM_CURSOR_BLOCK = 0,
    WEZTERM_CURSOR_UNDERLINE = 1,
    WEZTERM_CURSOR_BAR = 2,
} WezTermCursorStyle;

void wezterm_config_set_cursor_style(WezTermConfig* config, WezTermCursorStyle style);
void wezterm_config_set_cursor_blink(WezTermConfig* config, bool blink);

WezTermCursorStyle wezterm_config_get_cursor_style(const WezTermConfig* config);
bool wezterm_config_get_cursor_blink(const WezTermConfig* config);

/* Tab bar configuration */
void wezterm_config_set_hide_tab_bar_if_only_one_tab(WezTermConfig* config, bool hide);
void wezterm_config_set_tab_bar_at_bottom(WezTermConfig* config, bool bottom);
void wezterm_config_set_use_fancy_tab_bar(WezTermConfig* config, bool fancy);

bool wezterm_config_get_hide_tab_bar_if_only_one_tab(const WezTermConfig* config);
bool wezterm_config_get_tab_bar_at_bottom(const WezTermConfig* config);
bool wezterm_config_get_use_fancy_tab_bar(const WezTermConfig* config);

/* Behavior configuration */
void wezterm_config_set_check_for_updates(WezTermConfig* config, bool check);
void wezterm_config_set_native_fullscreen(WezTermConfig* config, bool native);
void wezterm_config_set_confirm_close_process(WezTermConfig* config, bool confirm);

bool wezterm_config_get_check_for_updates(const WezTermConfig* config);
bool wezterm_config_get_native_fullscreen(const WezTermConfig* config);
bool wezterm_config_get_confirm_close_process(const WezTermConfig* config);

/* ========== System Detection API ========== */

/**
 * Detect current system appearance
 * Returns: "light", "dark", "high_contrast_light", or "high_contrast_dark"
 */
const char* wezterm_detect_appearance(void);

/**
 * Detect system accent color
 * Returns: "blue", "purple", "pink", "red", "orange", "yellow", "green", or "graphite"
 */
const char* wezterm_detect_accent_color(void);

/**
 * Check if reduce motion is enabled
 */
bool wezterm_detect_reduced_motion(void);

/**
 * Check if reduce transparency is enabled
 */
bool wezterm_detect_reduce_transparency(void);

/**
 * Check if high contrast is enabled
 */
bool wezterm_detect_high_contrast(void);

/**
 * Get system font smoothing level (0-3)
 */
int32_t wezterm_detect_font_smoothing(void);

/* ========== Color Scheme API ========== */

/**
 * Get list of available color schemes
 * Returns: Array of scheme names (NULL-terminated)
 */
const char** wezterm_get_available_color_schemes(size_t* count);

/**
 * Free color scheme list
 */
void wezterm_free_color_scheme_list(const char** list, size_t count);

/**
 * Get color scheme by name
 */
WezTermColorScheme* wezterm_get_color_scheme(const char* name);

/**
 * Free color scheme
 */
void wezterm_color_scheme_free(WezTermColorScheme* scheme);

/**
 * Get color from scheme
 */
WezTermColor wezterm_color_scheme_get_ansi(const WezTermColorScheme* scheme, uint8_t index);
WezTermColor wezterm_color_scheme_get_foreground(const WezTermColorScheme* scheme);
WezTermColor wezterm_color_scheme_get_background(const WezTermColorScheme* scheme);

/* ========== Terminal API ========== */

/**
 * Create a new terminal instance
 */
WezTermTerminal* wezterm_terminal_new(uint16_t cols, uint16_t rows);

/**
 * Free a terminal instance
 */
void wezterm_terminal_free(WezTermTerminal* term);

/**
 * Write data to terminal
 */
size_t wezterm_terminal_write(WezTermTerminal* term, const uint8_t* data, size_t len);

/**
 * Resize terminal
 */
void wezterm_terminal_resize(WezTermTerminal* term, uint16_t cols, uint16_t rows);

/**
 * Get terminal dimensions
 */
void wezterm_terminal_get_size(const WezTermTerminal* term, uint16_t* cols, uint16_t* rows);

/**
 * Get cursor position
 */
void wezterm_terminal_get_cursor(const WezTermTerminal* term, uint16_t* col, uint16_t* row);

/**
 * Get terminal title
 */
const char* wezterm_terminal_get_title(const WezTermTerminal* term);

/* ========== String Utilities ========== */

/**
 * Free a string allocated by Rust
 */
void wezterm_string_free(char* s);

/**
 * Duplicate a string (must be freed with wezterm_string_free)
 */
char* wezterm_string_dup(const char* s);

/* ========== Application Control ========== */

/**
 * Open new window
 */
void wezterm_open_new_window(void);

/**
 * Open new tab
 */
void wezterm_open_new_tab(void);

/**
 * Open new tab with specific path
 */
void wezterm_open_new_tab_with_path(const char* path);

/**
 * Reload configuration
 */
void wezterm_reload_configuration(void);

/**
 * Show preferences window
 */
void wezterm_show_preferences(void);

/**
 * Get application version
 */
const char* wezterm_get_version(void);

#ifdef __cplusplus
}
#endif

#endif /* WEZTERM_BRIDGE_H */
