// Copyright 2022 The Druid Authors.
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

//! Handling of platform and integration events at the widget level.
//!
//! Note: arguably this module should be renamed, perhaps we should use
//! "event" for this level and maybe "message" at the View level.

use glazier::{KeyEvent, Modifiers, MouseButton, MouseButtons, Scale, TimerToken};
use vello::kurbo::{Point, Rect, Size, Vec2};

#[derive(Debug, Clone)]
// I am documenting them as pointers as the definitions have been changed in glazier
pub enum Event {
    /// Called when the window's [`Scale`] changes.
    ///
    /// This information can be used to switch between different resolution image assets.
    ///
    /// [`Scale`]: crate::Scale
    WindowScale(Scale),
    /// Called on the root widget when the window size changes.
    ///
    /// Discussion: it's not obvious this should be propagated to user
    /// widgets. It *is* propagated through the RootWidget and handled
    /// in the WindowPod, but after that it might be considered better
    /// to just handle it in `layout`.
    WindowSize(Size),
    /// Called when a pointer button is pressed.
    MouseDown(MouseEvent),
    /// Called when a mouse button is released.
    MouseUp(MouseEvent),
    /// Called when the mouse is moved.
    ///
    /// The `MouseMove` event is propagated to the active widget, if
    /// there is one, otherwise to hot widgets (see [`HotChanged`]).
    /// If a widget loses its hot status due to `MouseMove` then that specific
    /// `MouseMove` event is also still sent to that widget. However a widget
    /// can lose its hot status even without a `MouseMove` event, so make
    /// sure to also handle [`HotChanged`] if you care about the hot status.
    ///
    /// The `MouseMove` event is also the primary mechanism for widgets
    /// to set a cursor, for example to an I-bar inside a text widget. A
    /// simple tactic is for the widget to unconditionally call
    /// [`set_cursor`] in the MouseMove handler, as `MouseMove` is only
    /// propagated to active or hot widgets.
    ///
    /// [`HotChanged`]: LifeCycle::HotChanged
    /// [`set_cursor`]: crate::EventCtx::set_cursor
    MouseMove(MouseEvent),
    /// Called when the mouse wheel or trackpad is scrolled.
    MouseWheel(MouseEvent),
    MouseLeft(),
    /// Called when a key is pressed.
    KeyDown(KeyEvent),
    /// Called when a key is released.
    ///
    /// Because of repeat, there may be a number `KeyDown` events before
    /// a corresponding `KeyUp` is sent.
    KeyUp(KeyEvent),
    /// Called on a timer event.
    ///
    /// Request a timer event through [`EventCtx::request_timer`]. That will
    /// cause a timer event later.
    ///
    /// Note that timer events from other widgets may be delivered as well. Use
    /// the token returned from the `request_timer` call to filter events more
    /// precisely.
    ///
    /// [`EventCtx::request_timer`]: crate::EventCtx::request_timer
    Timer(TimerToken),
    /// Called at the beginning of a new animation frame.
    ///
    /// On the first frame when transitioning from idle to animating, `interval`
    /// will be 0. (This logic is presently per-window but might change to
    /// per-widget to make it more consistent). Otherwise it is in nanoseconds.
    ///
    /// Receiving `AnimFrame` does not inherently mean a `paint` invocation will follow.
    /// If you want something actually painted you need to explicitly call [`request_paint`]
    /// or [`request_paint_rect`].
    ///
    /// If you do that, then the `paint` method will be called shortly after this event is finished.
    /// As a result, you should try to avoid doing anything computationally
    /// intensive in response to an `AnimFrame` event: it might make Druid miss
    /// the monitor's refresh, causing lag or jerky animation.
    ///
    /// You can request an `AnimFrame` via [`request_anim_frame`].
    ///
    /// [`request_paint`]: crate::EventCtx::request_paint
    /// [`request_paint_rect`]: crate::EventCtx::request_paint_rect
    /// [`request_anim_frame`]: crate::EventCtx::request_anim_frame
    AnimFrame(u64),
    TargetedAccessibilityAction(accesskit::ActionRequest),
}

#[derive(Debug, Clone)]
pub struct MouseEvent {
    /// The position of the mouse in the coordinate space of the receiver.
    pub pos: Point,
    /// The position of the mose in the window coordinate space.
    pub window_pos: Point,
    /// Mouse buttons being held down during a move or after a click event.
    /// Thus it will contain the `button` that triggered a mouse-down event,
    /// and it will not contain the `button` that triggered a mouse-up event.
    pub buttons: MouseButtons,
    /// Keyboard modifiers at the time of the event.
    pub mods: Modifiers,
    /// The number of mouse clicks associated with this event. This will always
    /// be `0` for a mouse-up and mouse-move events.
    pub count: u8,
    /// Focus is `true` on macOS when the mouse-down event (or its companion mouse-up event)
    /// with `MouseButton::Left` was the event that caused the window to gain focus.
    ///
    /// This is primarily used in relation to text selection.
    /// If there is some text selected in some text widget and it receives a click
    /// with `focus` set to `true` then the widget should gain focus (i.e. start blinking a cursor)
    /// but it should not change the text selection. Text selection should only be changed
    /// when the click has `focus` set to `false`.
    pub focus: bool,
    /// The button that was pressed down in the case of mouse-down,
    /// or the button that was released in the case of mouse-up.
    /// This will always be `MouseButton::None` in the case of mouse-move.
    pub button: MouseButton,
    /// The wheel movement.
    ///
    /// The polarity is the amount to be added to the scroll position,
    /// in other words the opposite of the direction the content should
    /// move on scrolling. This polarity is consistent with the
    /// deltaX and deltaY values in a web [WheelEvent].
    ///
    /// [WheelEvent]: https://w3c.github.io/uievents/#event-type-wheel
    pub wheel_delta: Vec2,
}

#[derive(Debug)]
pub enum LifeCycle {
    HotChanged(bool),
    ViewContextChanged(ViewContext),
    TreeUpdate,
}

#[derive(Debug)]
pub struct ViewContext {
    pub window_origin: Point,
    pub clip: Rect,
    pub mouse_position: Option<Point>,
}

impl<'a> From<&'a glazier::MouseEvent> for MouseEvent {
    fn from(src: &glazier::MouseEvent) -> MouseEvent {
        let glazier::MouseEvent {
            pos,
            buttons,
            mods,
            count,
            focus,
            button,
            wheel_delta,
        } = src;
        MouseEvent {
            pos: *pos,
            window_pos: *pos,
            buttons: *buttons,
            mods: *mods,
            count: *count,
            focus: *focus,
            button: *button,
            wheel_delta: *wheel_delta,
        }
    }
}

impl ViewContext {
    pub fn translate_to(&self, new_origin: Point) -> ViewContext {
        let translate = new_origin.to_vec2();
        ViewContext {
            window_origin: self.window_origin + translate,
            clip: self.clip - translate,
            mouse_position: self.mouse_position.map(|p| p - translate),
        }
    }
}
