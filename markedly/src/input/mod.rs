//! Systems for handling user input.

use nalgebra::{Point2, Vector2};

use {Ui, ComponentId, ComponentFlow};

/// Handles user input, raising events on components and storing current input information.
pub struct Input {
    hovering_over: Option<ComponentId>,
}

impl Input {
    /// Creates a new UI input handler.
    pub fn new() -> Self {
        Input {
            hovering_over: None,
        }
    }

    /// Returns true if the cursor is currently over a UI element that captures cursor movement.
    pub fn is_cursor_over_ui(&self) -> bool {
        self.hovering_over.is_some()
    }

    /// Handles cursor movement.
    pub fn handle_cursor_moved(
        &mut self, position: Point2<f32>, ui: &mut Ui,
    ) {
        let mut flow = ComponentFlow::new(ui.target_size());
        let new_hovering = find_at_position(
            position, ui, ui.root_id(), Point2::new(0.0, 0.0), Vector2::new(0.0, 0.0), &mut flow,
        );

        if let Some(new_hovering) = new_hovering {
            // If the thing we're hovering over is a new thing, we need to notify it
            if self.hovering_over.map(|v| v != new_hovering).unwrap_or(true) {
                let component = ui.get_mut(new_hovering).unwrap();
                component.needs_render_update |=
                    component.class.hover_start_event(&mut component.event_sink);
            }
        }

        if let Some(hovering_over) = self.hovering_over {
            // If the thing we're hovering over is a new thing, we need to notify the old one
            if new_hovering.map(|v| v != hovering_over).unwrap_or(true) {
                let component = ui.get_mut(hovering_over).unwrap();
                component.needs_render_update |=
                    component.class.hover_end_event(&mut component.event_sink);
            }
        }

        self.hovering_over = new_hovering;
    }

    /// Handles the start of a cursor or touch drag.
    pub fn handle_drag_started(
        &mut self, _position: Point2<f32>, _ui: &mut Ui,
    ) {
    }

    /// Handles the end of a cursor or touch drag.
    pub fn handle_drag_ended(
        &mut self, position: Point2<f32>, ui: &mut Ui,
    ) {
        let mut flow = ComponentFlow::new(ui.target_size());
        if let Some(component_id) = find_at_position(
            position, ui, ui.root_id(), Point2::new(0.0, 0.0), Vector2::new(0.0, 0.0), &mut flow,
        ) {
            let component = ui.get_mut(component_id).unwrap();
            component.class.pressed_event(&mut component.event_sink);
        }
    }
}

fn find_at_position(
    position: Point2<f32>, ui: &Ui, id: ComponentId,
    computed_parent_position: Point2<f32>, parent_size: Vector2<f32>,
    parent_flow: &mut ComponentFlow,
) -> Option<ComponentId> {
    let component = ui.get(id).unwrap();
    let computed_position = computed_parent_position +
        component.compute_position(parent_size, parent_flow).coords;
    let computed_size = component.compute_size(parent_size);

    // If the position isn't over us, it also won't be over any children, so just return none
    if position.x < computed_position.x ||
        position.y < computed_position.y ||
        position.x > computed_position.x + computed_size.x ||
        position.y > computed_position.y + computed_size.y {
        return None
    }

    // If this component doesn't capture input, we still need to check children, but we can't
    // return this one.
    let mut found_id = if component.class.is_capturing_cursor() {
        Some(id)
    } else {
        None
    };

    // Go through all children, if any of them find a hit, replace the ID we found, we want to find
    // the last one that matches because it's the one rendered on top. The function will
    // recursively find the deepest matching child like this.
    let mut flow = ComponentFlow::new(computed_size);
    for child_id in &component.children {
        if let Some(id) = find_at_position(
            position, ui, *child_id, computed_position, computed_size, &mut flow,
        ) {
            found_id = Some(id);
        }
    }

    found_id
}
