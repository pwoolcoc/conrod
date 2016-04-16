//! Handles all of the global input events and state.
//! The core of this module is the `GlobalInput` struct. It is responsible for aggregating
//! and interpreting raw input events into high-level semantic events.

use events::{InputState, UiEvent, MouseClick, MouseDrag, Scroll, InputProvider};
use input::MouseButton;
use position::{Point, Scalar};
use widget::Index;

/// Global input event handler that also implements `InputProvider`. The `Ui` passes all events
/// to it's `GlobalInput` instance, which aggregates and interprets the events to provide
/// so-called 'high-level' events to widgets. This input gets reset after every update by the `Ui`.
pub struct GlobalInput {
    /// The `InputState` as it was at the end of the last update cycle.
    pub start_state: InputState,
    /// The most recent `InputState`, with updates from handling all the events
    /// this update cycle
    pub current_state: InputState,
    events: Vec<UiEvent>,
    drag_threshold: Scalar,
}

/// Iterator over global `UiEvent`s. Unlike the `WidgetInputEventIterator`, this will
/// never filter out any events, and all coordinates will be reative to the (0,0) origin
/// of the window.
pub type GlobalInputEventIterator<'a> = ::std::slice::Iter<'a, UiEvent>;

impl<'a> InputProvider<'a> for GlobalInput {
    type Events = GlobalInputEventIterator<'a>;

    fn all_events(&'a self) -> Self::Events {
        self.events.iter()
    }

    fn current_state(&'a self) -> &'a InputState {
        &self.current_state
    }

    fn mouse_button_down(&self, button: MouseButton) -> Option<Point> {
         self.current_state().mouse_buttons.get(button).map(|_| {
             self.mouse_position()
         })
    }
}

impl GlobalInput {

    /// Returns a fresh new `GlobalInput`
    pub fn new(drag_threshold: Scalar) -> GlobalInput {
        GlobalInput{
            events: Vec::new(),
            drag_threshold: drag_threshold,
            start_state: InputState::new(),
            current_state: InputState::new(),
        }
    }

    /// Adds a new event and updates the internal state.
    pub fn push_event(&mut self, event: UiEvent) {
        use input::Input::{Release, Move};
        use input::Motion::MouseRelative;
        use input::Motion::MouseScroll;
        use input::Button::Mouse;

        let maybe_new_event = match event {
            UiEvent::Raw(Release(Mouse(button))) => self.handle_mouse_release(button),
            UiEvent::Raw(Move(MouseRelative(x, y))) => self.handle_mouse_move([x, y]),
            UiEvent::Raw(Move(MouseScroll(x, y))) => self.mouse_scroll(x, y),
            _ => None
        };

        self.current_state.update(&event);
        self.events.push(event);
        if let Some(new_event) = maybe_new_event {
            self.push_event(new_event);
        }
    }

    /// Called at the end of every update cycle in order to prepare the `GlobalInput` to
    /// handle events for the next one.
    pub fn reset(&mut self) {
        self.events.clear();
        self.start_state = self.current_state.clone();
    }

    /// Returns the most up to date position of the mouse
    pub fn mouse_position(&self) -> Point {
        self.current_state.mouse_position
    }

    /// Returns the input state as it was after the last update
    pub fn starting_state(&self) -> &InputState {
        &self.start_state
    }

    /// Returns the most up to date info on which widget is capturing the mouse
    pub fn currently_capturing_mouse(&self) -> Option<Index> {
        self.current_state.widget_capturing_mouse
    }

    /// Returns the most up to date info on which widget is capturing the keyboard
    pub fn currently_capturing_keyboard(&self) -> Option<Index> {
        self.current_state.widget_capturing_keyboard
    }


    fn mouse_scroll(&self, x: f64, y: f64) -> Option<UiEvent> {
        Some(UiEvent::Scroll(Scroll{
            x: x,
            y: y,
            modifiers: self.current_state.modifiers
        }))
    }

    fn handle_mouse_move(&self, move_to: Point) -> Option<UiEvent> {
        self.current_state.mouse_buttons.pressed_button().and_then(|btn_and_point| {
            if self.is_drag(btn_and_point.1, move_to) {
                Some(UiEvent::MouseDrag(MouseDrag{
                    button: btn_and_point.0,
                    start: btn_and_point.1,
                    end: move_to,
                    in_progress: true,
                    modifier: self.current_state.modifiers
                }))
            } else {
                None
            }
        })
    }

    fn handle_mouse_release(&self, button: MouseButton) -> Option<UiEvent> {
        self.current_state.mouse_buttons.get(button).map(|point| {
            if self.is_drag(point, self.current_state.mouse_position) {
                UiEvent::MouseDrag(MouseDrag{
                    button: button,
                    start: point,
                    end: self.current_state.mouse_position,
                    modifier: self.current_state.modifiers,
                    in_progress: false
                })
            } else {
                UiEvent::MouseClick(MouseClick {
                    button: button,
                    location: point,
                    modifier: self.current_state.modifiers
                })
            }
        })
    }

    fn is_drag(&self, a: Point, b: Point) -> bool {
        distance_between(a, b) > self.drag_threshold
    }
}

fn distance_between(a: Point, b: Point) -> Scalar {
    let dx_2 = (a[0] - b[0]).powi(2);
    let dy_2 = (a[1] - b[1]).powi(2);
    (dx_2 + dy_2).abs().sqrt()
}