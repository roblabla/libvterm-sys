# **************************************************************************** #
#                                                                              #
#                                                         :::      ::::::::    #
#    vterm.rs                                           :+:      :+:    :+:    #
#                                                     +:+ +:+         +:+      #
#    By: roblabla <robinlambertz+dev@gmail.c>       +#+  +:+       +#+         #
#                                                 +#+#+#+#+#+   +#+            #
#    Created: 2015/07/26 15:31:47 by roblabla          #+#    #+#              #
#    Updated: 2015/08/01 17:05:43 by roblabla         ###   ########.fr        #
#                                                                              #
# **************************************************************************** #

use vterm_keycodes::*;
use libc::{c_void, c_char, c_int, c_long, uint8_t, uint32_t, size_t};
use std;

pub type VTerm = c_void;
pub type VTermState = c_void;
pub type VTermScreen = c_void;

#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct VTermPos {
    pub row: c_int,
    pub col: c_int,
}

pub fn vterm_pos_cmp(a: VTermPos, b: VTermPos) -> c_int {
    if a.row == b.row { a.col - b.col } else { a.row - b.row }
}

#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct VTermRect {
    pub start_row: c_int,
    pub end_row: c_int,
    pub start_col: c_int,
    pub end_col: c_int,
}

pub fn vterm_rect_contains(r: VTermRect, p: VTermPos) -> bool {
    p.row >= r.start_row && p.row < r.end_row &&
    p.col >= r.start_row && p.col < r.end_col
}

pub fn vterm_rect_move(rect: &mut VTermRect, row_delta: c_int, col_delta: c_int) {
    rect.start_row += row_delta; rect.end_row += row_delta;
    rect.start_col += col_delta; rect.end_col += col_delta;
}

#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct VTermColor {
    pub red: uint8_t,
    pub green: uint8_t,
    pub blue: uint8_t,
}

#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub enum VTermValueType {
    VTERM_VALUETYPE_BOOL = 1,
    VTERM_VALUETYPE_INT,
    VTERM_VALUETYPE_STRING,
    VTERM_VALUETYPE_COLOR,
}

// TODO : Calculate size of VTermValue at compile-time
#[repr(C)]
#[derive(Debug)]
pub struct VTermValue {
    ptr: *mut c_char,
}

impl VTermValue {
    pub fn boolean(&self) -> c_int {
        unsafe { std::mem::transmute_copy(self) }
    }
    pub fn number(&self) -> c_int {
        unsafe { std::mem::transmute_copy(self) }
    }
    pub fn string(&self) -> *mut c_char {
        unsafe { std::mem::transmute_copy(self) }
    }
    pub fn color(&self) -> VTermColor {
        unsafe { std::mem::transmute_copy(self) }
    }
}

#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub enum VTermAttr {
    VTERM_ATTR_BOLD = 1,
    VTERM_ATTR_UNDERLINE,
    VTERM_ATTR_ITALIC,
    VTERM_ATTR_BLINK,
    VTERM_ATTR_REVERSE,
    VTERM_ATTR_STRIKE,
    VTERM_ATTR_FONT,
    VTERM_ATTR_FOREGROUND,
    VTERM_ATTR_BACKGROUND,
}

#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub enum VTermProp {
    VTERM_PROP_CURSORVISIBLE = 1,
    VTERM_PROP_CURSORBLINK,
    VTERM_PROP_ALTSCREEN,
    VTERM_PROP_TITLE,
    VTERM_PROP_ICONNAME,
    VTERM_PROP_REVERSE,
    VTERM_PROP_CURSORSHAPE,
    VTERM_PROP_MOUSE,
}

#[repr(C)]
#[derive(Debug)]
pub struct VTermGlyphInfo; // TODO : bitfields are a bitch.

#[repr(C)]
#[derive(Debug)]
pub struct VTermLineInfo; // TODO : bitfields are a bitch.

#[repr(C)]
//#[derive(Debug)]
// Function types don't implement Debug
pub struct VTermAllocatorFunctions {
    pub malloc: extern fn(size: size_t, allocdata: *mut c_void) -> *mut c_void,
    pub free: extern fn(ptr: *mut c_void, allocdata: *mut c_void),
}

#[link(name = "vterm")]
extern {
    pub fn vterm_new(rows: c_int, cols: c_int) -> *mut VTerm;
    pub fn vterm_new_with_allocator(rows: c_int,
                                    cols: c_int,
                                    funcs: *mut VTermAllocatorFunctions,
                                    allocdata: *mut c_void) -> *mut VTerm;
    pub fn vterm_free(vt: *mut VTerm);

    pub fn vterm_get_size(vt: *const VTerm,
                          rowsp: *mut c_int,
                          colsp: *mut c_int);
    pub fn vterm_set_size(vt: *mut VTerm, rows: c_int, cols: c_int);

    pub fn vterm_get_utf8(vt: *const VTerm) -> c_int;
    pub fn vterm_set_utf8(vt: *mut VTerm, is_utf8: c_int);

    pub fn vterm_input_write(vt: *mut VTerm,
                             bytes: *const c_char,
                             len: size_t) -> size_t;

    pub fn vterm_output_get_buffer_size(vt: *const VTerm) -> size_t;
    pub fn vterm_output_get_buffer_current(vt: *const VTerm) -> size_t;
    pub fn vterm_output_get_buffer_remaining(vt: *const VTerm) -> size_t;

    pub fn vterm_output_read(vt: *mut VTerm,
                             buffer: *mut c_char,
                             len: size_t) -> size_t;

    pub fn vterm_keyboard_unichar(vt: *mut VTerm,
                                  c: uint32_t,
                                  modifier: VTermModifier);
    pub fn vterm_keyboard_key(vt: *mut VTerm,
                              key: VTermKey,
                              modifier: VTermModifier);

    pub fn vterm_mouse_move(vt: *mut VTerm,
                            row: c_int,
                            col: c_int,
                            modifier: VTermModifier);
    pub fn vterm_mouse_button(vt: *mut VTerm,
                              button: c_int,
                              pressed: c_int,
                              modifier: VTermModifier);
}

// ------------
// Parser layer
// ------------

/* Flag to indicate non-final subparameters in a single CSI parameter.
 * Consider
 *   CSI 1;2:3:4;5a
 * 1 4 and 5 are final.
 * 2 and 3 are non-final and will have this bit set
 *
 * Don't confuse this with the final byte of the CSI escape; 'a' in this case.
 */

// TODO : CSI

#[repr(C)]
//#[derive(Debug)]
pub struct VTermParserCallbacks {
    pub text: extern fn(*const c_char, size_t, *mut c_void) -> c_int,
    pub control: extern fn(c_char, *mut c_void) -> c_int,
    pub escape: extern fn(*const c_char, size_t, *mut c_void) -> c_int,
    pub csi: extern fn(*const c_char, *const c_long, c_int, *const c_char, c_char, *mut c_void) -> c_int,
    pub osc: extern fn(*const c_char, size_t, *mut c_void) -> c_int,
    pub dcs: extern fn(*const c_char, size_t, *mut c_void) -> c_int,
    pub resize: extern fn(c_int, c_int, *mut c_void) -> c_int,
}

#[link(name = "vterm")]
extern {
    pub fn vterm_parser_set_callbacks(vt: *mut VTerm,
                                      callbacks: *const VTermParserCallbacks,
                                      user: *mut c_void);
    pub fn vterm_parser_get_cbdata(vt: *mut VTerm) -> *mut c_void;
}

// -----------
// State layer
// -----------

#[repr(C)]
// #[derive(Debug)]
pub struct VTermStateCallbacks {
    pub putglyph: extern fn(*mut VTermGlyphInfo, VTermPos, *mut c_void) -> c_int,
    pub movecursor: extern fn(VTermPos, VTermPos, c_int, *mut c_void) -> c_int,
    pub scrollrect: extern fn(VTermRect, c_int, c_int, *mut c_void) -> c_int,
    pub moverect: extern fn(VTermRect, VTermRect, *mut c_void) -> c_int,
    pub erase: extern fn(VTermRect, c_int, *mut c_void) -> c_int,
    pub initpen: extern fn(*mut c_void) -> c_int,
    pub setpenattr: extern fn(VTermAttr, *mut VTermValue, *mut c_void) -> c_int,
    pub settermprop: extern fn(VTermProp, *mut VTermValue, *mut c_void) -> c_int,
    pub bell: extern fn(*mut c_void) -> c_int,
    pub resize: extern fn(c_int, c_int, *mut VTermPos, *mut c_void) -> c_int,
    pub setlineinfo: extern fn(c_int, *const VTermLineInfo, *const VTermLineInfo, *mut c_void) -> c_int,
}

#[link(name = "vterm")]
extern {
    pub fn vterm_obtain_state(vt: *mut VTerm) -> *mut VTermState;
    pub fn vterm_state_set_callbacks(state: *mut VTermState,
                                     callbacks: *const VTermStateCallbacks,
                                     user: *mut c_void);
    pub fn vterm_state_get_cbdata(state: *mut VTermState) -> *mut c_void;

    pub fn vterm_state_reset(state: *mut VTermState, hard: c_int);
    pub fn vterm_state_get_cursrpos(state: *const VTermState,
                                    cursorpos: *mut VTermPos);
    pub fn vterm_state_get_default_colors(state: *const VTermState,
                                          default_fg: *mut VTermColor,
                                          default_bg: *mut VTermColor);
    pub fn vterm_state_get_palette_color(state: *const VTermState,
                                         index: c_int,
                                         col: *mut VTermColor);
    pub fn vterm_state_set_default_colors(state: *mut VTermState,
                                          default_fg: *const VTermColor,
                                          default_bg: *const VTermColor);
    pub fn vterm_state_set_palette_color(state: *mut VTermState,
                                         index: c_int,
                                         col: *const VTermColor);
    pub fn vterm_state_set_bold_highbright(state: *mut VTermState,
                                           bold_is_highbright: c_int);
    pub fn vterm_state_get_penattr(state: *const VTermState,
                                   attr: VTermAttr,
                                   val: *mut VTermValue) -> c_int;
    pub fn vterm_state_set_termprop(state: *mut VTermState,
                                    prop: VTermProp,
                                    val: *mut VTermValue) -> c_int;
    pub fn vterm_state_get_lineinfo(state: *const VTermState,
                                    row: c_int) -> *const VTermLineInfo;
}

// ------------
// Screen layer
// ------------

const VTERM_MAX_CHARS_PER_CELL : usize = 6;

// TODO : VTermAttrs
#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct VTermScreenCell {
    pub chars: [uint32_t; VTERM_MAX_CHARS_PER_CELL],
    pub width: c_char,
    pub attrs: i32, // TODO !
    pub fg: VTermColor,
    pub bg: VTermColor,
}


#[repr(C)]
// #[derive(Debug)]
pub struct VTermScreenCallbacks {
    pub damage: extern fn(VTermRect, *mut c_void) -> c_int,
    pub moverect: extern fn(VTermRect, VTermRect, *mut c_void) -> c_int,
    pub movecursor: extern fn(VTermPos, VTermPos, c_int, *mut c_void) -> c_int,
    pub settermprop: extern fn(VTermProp, *mut VTermValue, *mut c_void) -> c_int,
    pub bell: extern fn(*mut c_void) -> c_int,
    pub resize: extern fn(c_int, c_int, *mut c_void) -> c_int,
    pub sb_pushline: extern fn(c_int, *const VTermScreenCell, *mut c_void) -> c_int,
    pub sb_popline: extern fn(c_int, *mut VTermScreenCell, *mut c_void) -> c_int,
}

#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub enum VTermDamageSize {
    VTERM_DAMAGE_CELL,    /* every cell */
    VTERM_DAMAGE_ROW,     /* entire rows */
    VTERM_DAMAGE_SCREEN,  /* entire screen */
    VTERM_DAMAGE_SCROLL,  /* entire screen + scrollrect */
}

#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub enum VTermAttrMask {
    VTERM_ATTR_BOLD_MASK       = 1 << 0,
    VTERM_ATTR_UNDERLINE_MASK  = 1 << 1,
    VTERM_ATTR_ITALIC_MASK     = 1 << 2,
    VTERM_ATTR_BLINK_MASK      = 1 << 3,
    VTERM_ATTR_REVERSE_MASK    = 1 << 4,
    VTERM_ATTR_STRIKE_MASK     = 1 << 5,
    VTERM_ATTR_FONT_MASK       = 1 << 6,
    VTERM_ATTR_FOREGROUND_MASK = 1 << 7,
    VTERM_ATTR_BACKGROUND_MASK = 1 << 8,
}

#[link(name = "vterm")]
extern {
    pub fn vterm_obtain_screen(vt: *mut VTerm) -> *mut VTermScreen;
    pub fn vterm_screen_get_cbdata(screen: *mut VTermScreen) -> *mut c_void;

    pub fn vterm_screen_enable_altscreen(screen: *mut VTermScreen,
                                         altscreen: c_int);
    pub fn vterm_screen_flush_damage(screen: *mut VTermScreen);
    pub fn vterm_screen_set_damage_merge(screen: *mut VTermScreen,
                                         size: VTermDamageSize);

    pub fn vterm_screen_reset(screen: *mut VTermScreen, hard: c_int);

    /* Neither of these functions NUL-terminate the buffer */
    pub fn vterm_screen_get_chars(screen: *const VTermScreen,
                                  chars: *mut uint32_t,
                                  len: size_t,
                                  rect: VTermRect) -> size_t;
    pub fn vterm_screen_get_text(screen: *const VTermScreen,
                                 string: *mut c_char,
                                 len: size_t,
                                 rect: VTermRect) -> size_t;

    pub fn vterm_screen_get_attrs_extent(screen: *const VTermScreen,
                                         extent: *mut VTermRect,
                                         pos: VTermPos,
                                         attrs: VTermAttrMask) -> c_int;
    pub fn vterm_screen_get_cell(screen: *const VTermScreen,
                                 pos: VTermPos,
                                 cell: *mut VTermScreenCell);
    pub fn vterm_screen_is_eol(screen: *const VTermScreen, pos: VTermPos);
}

// ---------
// Utilities
// ---------

#[link(name = "vterm")]
extern {
    pub fn vterm_get_attr_type(attr: VTermAttr) -> VTermValueType;
    pub fn vterm_get_prop_type(prop: VTermProp) -> VTermValueType;

    pub fn vterm_scroll_rect(rect: VTermRect,
                             downward: c_int,
                             rightward: c_int,
                             moverect: extern fn(VTermRect,
                                                 VTermRect,
                                                 *mut c_void) -> c_int,
                             eraserect: extern fn(VTermRect,
                                                  c_int,
                                                  *mut c_void) -> c_int,
                             user: *mut c_void);
    pub fn vterm_copy_cell(dest: VTermRect,
                           src: VTermRect,
                           copycell: extern fn (VTermPos,
                                                VTermPos,
                                                *mut c_void) -> c_int,
                           user: *mut c_void);
}
