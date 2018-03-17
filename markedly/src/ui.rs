use nalgebra::{Vector2};
use metrohash::{MetroHashMap, MetroHashSet};

use class::{ComponentClasses};
use scripting::{ScriptTable, ScriptRuntime};
use template::{Style, Template, ComponentTemplate};
use {Component, EventSink, Error};

/// A self-contained UI, to be rendered to a single target, be that full screen, in-world, or used
/// in some other way.
pub struct Ui {
    style: Style,
    target_size: Vector2<f32>,
    root_id: ComponentId,

    components: MetroHashMap<ComponentId, Component>,
    next_id: ComponentId,

    tree_roots: MetroHashSet<ComponentId>,
}

impl Ui {
    /// Creates a new UI from a root template.
    pub fn new(
        template: &Template, model: Option<&ScriptTable>,
        style: Style, target_size: Vector2<f32>, context: &Context,
    ) -> Result<(Self, Tree), Error> {
        let mut ui = Ui {
            style,
            target_size,
            root_id: ComponentId(0),

            components: MetroHashMap::default(),
            next_id: ComponentId(0),

            tree_roots: MetroHashSet::default(),
        };

        // Prepare the scripting engine with the model data
        let default_table = ScriptTable::new();
        let model = model.unwrap_or(&default_table);
        context.runtime.set_model(&model)?;

        // Create the root component from the template
        let event_sink = EventSink::new();
        ui.root_id = ui.load_component(&template.root, event_sink.clone(), context)?;

        let root = ui.root_id;
        Ok((ui, Tree { root, event_sink, }))
    }

    pub fn target_size(&self) -> Vector2<f32> {
        self.target_size
    }

    /// Gets a component from its ID.
    pub fn get(&self, id: ComponentId) -> Option<&Component> {
        self.components.get(&id)
    }

    /// Gets a component as mutable from its ID.
    pub fn get_mut(&mut self, id: ComponentId) -> Option<&mut Component> {
        self.components.get_mut(&id)
    }

    /// Gets the root component's ID.
    pub fn root_id(&self) -> ComponentId {
        self.root_id
    }

    /// Inserts a template into the UI as a child of the first found component that has the given
    /// style class.
    pub fn insert_template(
        &mut self,
        template: &Template, model: Option<&ScriptTable>,
        style_class: &str,
        context: &Context,
    ) -> Result<Tree, Error> {
        // Find the first component that has a style class matching what we were asked for
        let mut found_parent_id = None;
        for (key, component) in &self.components {
            if let Some(ref component_style_class) = component.style_class {
                if component_style_class == style_class {
                    found_parent_id = Some(*key);
                }
            }
        }

        // Make sure we found something and retrieve some basic data we need
        let parent_id = found_parent_id
            .ok_or(format!("Unable to find component with style class {}", style_class))?;

        // Prepare the scripting engine with the model data
        let default_table = ScriptTable::new();
        let model = model.unwrap_or(&default_table);
        context.runtime.set_model(&model)?;

        // Recursively add the template
        let event_sink = EventSink::new();
        let id = self.load_component(&template.root, event_sink.clone(), context)?;

        // Add the component tree we just added to the children of the component we had found
        self.get_mut(parent_id).unwrap().children.push(id);

        Ok(Tree { root: id, event_sink, })
    }

    pub fn update_model(
        &mut self, tree: &Tree, model: &ScriptTable, context: &Context,
    ) -> Result<(), Error> {
        // Reloading everything isn't very efficient, it should be changed to
        // detect which model values components have been bound to and only update the
        // relevant ones
        context.runtime.set_model(&model)?;

        Self::update_component_recursive(
            &mut self.components, tree.root, &self.tree_roots, &self.style, context
        )?;

        Ok(())
    }

    pub(crate) fn mark_all_rendered(&mut self) {
        for (_key, value) in &mut self.components {
            value.needs_render_update = false;
        }
    }

    fn load_component(
        &mut self,
        template: &ComponentTemplate,
        event_sink: EventSink,
        context: &Context,
    ) -> Result<ComponentId, Error> {
        // Load the component itself from the template
        let mut component = Component::from_template(
            template, event_sink.clone(), &self.style, context,
        )?;
        let id = self.next_id;
        self.next_id.0 += 1;

        // Also load all the children
        for child in &template.children {
            let id = self.load_component(child, event_sink.clone(), context)?;
            component.children.push(id);
        }

        // Add the component itself
        self.components.insert(id, component);

        Ok(id)
    }

    fn update_component_recursive(
        components: &mut MetroHashMap<ComponentId, Component>, key: ComponentId,
        tree_roots: &MetroHashSet<ComponentId>,
        style: &Style, context: &Context,
    ) -> Result<(), Error> {
        for child_i in 0..components.get(&key).unwrap().children.len() {
            let child_id = components.get(&key).unwrap().children[child_i];

            // Do not go deeper if we're at an inserted template's root
            if !tree_roots.contains(&child_id) {
                Self::update_component_recursive(
                    components, child_id, tree_roots, style, context
                )?;
            }
        }

        components.get_mut(&key).unwrap().update_attributes(style, context)?;

        Ok(())
    }
}

/// The context UIs should be processed and rendered in, this defines the overall UI system's
/// configuration, such as what component classes are available and how the scripting runtime is
/// configured.
pub struct Context {
    pub classes: ComponentClasses,
    pub runtime: ScriptRuntime,
}

/// An ID pointing to a component in a UI.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ComponentId(pub i32);

/// An handle for a tree of components in a UI.
pub struct Tree {
    root: ComponentId,
    event_sink: EventSink,
}

impl Tree {
    pub fn event_sink(&self) -> &EventSink {
        &self.event_sink
    }
}
