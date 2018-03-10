extern crate ggez;
extern crate markedly;
extern crate markedly_ggez;

use std::env;
use std::path;

use ggez::{Context, GameResult, GameError};
use ggez::conf::{Conf, WindowMode, WindowSetup};
use ggez::event::{self, EventHandler, MouseButton, MouseState};
use ggez::graphics::{self, Point2, Vector2};

use markedly::class::{ComponentClasses};
use markedly::input::{Input};
use markedly::scripting::{ScriptRuntime};
use markedly::template::{Template, Style};
use markedly::{Context as UiContext, Ui, Tree};

use markedly_ggez::{GgezRenderer, GgezCache, emtg};

fn main() {
    // Set up the ggez context
    let mut c = Conf::new();
    c.window_mode = WindowMode {
        width: 1280,
        height: 720,
        .. Default::default()
    };
    c.window_setup = WindowSetup {
        title: "Markedly Example".into(),
        .. Default::default()
    };
    let ctx = &mut Context::load_from_conf("example", "markedly", c).unwrap();

    // We add the CARGO_MANIFEST_DIR/resources do the filesystems paths so we we look in the cargo
    // project for files.
    if let Ok(manifest_dir) = env::var("CARGO_MANIFEST_DIR") {
        let mut path = path::PathBuf::from(manifest_dir);
        path.push("resources");
        ctx.filesystem.mount(&path, true);
    }

    // Initialize and run the game
    let result = MainState::new(ctx)
        .and_then(|mut s| event::run(ctx, &mut s));

    // Check if it ran successfully
    if let Err(e) = result {
        match e {
            GameError::UnknownError(text) => println!("Fatal:\n{}", text),
            e => println!("Fatal: {}", e)
        }
    } else {
        println!("Game exited cleanly");
    }
}


struct MainState {
    _ui_context: UiContext,
    ui: Ui,

    ui_input: Input,
    ui_cache: GgezCache,
    ui_root: Tree,
}

impl MainState {
    pub fn new(ctx: &mut Context) -> GameResult<Self> {
        let screen_size = Vector2::new(1280.0, 720.0);

        // Register all the component classes, this makes them available to be used in templates.
        let mut classes = ComponentClasses::new();
        classes.register::<markedly::class::ContainerClass>("container");
        classes.register::<markedly::class::ButtonClass>("button");

        // Set up the scripting runtime.
        // TODO: Here you can make custom helper functions available to templates.
        let runtime = ScriptRuntime::new();

        // The context is a bundle of the systems needed for a UI to function.
        let ui_context = UiContext { classes, runtime, };

        // This UI will make use of input. If your UI will not use input, for example if your UI is
        // an in-game screen, you don't need this.
        let ui_input = Input::new();

        // Set up the UI cache.
        // This will keep track of rendering data, as well as resources to be used by templates.
        let mut ui_cache = GgezCache::new();
        ui_cache.add_font("raleway", "/Raleway-Regular.ttf").map_err(emtg)?;
        ui_cache.add_font("cormorant", "/CormorantGaramond-Regular.ttf").map_err(emtg)?;

        // Load in a style template.
        // This defines some default styles and style classes to be used when displaying templates.
        let style = Style::from_reader(ctx.filesystem.open("/mark/_style.mark")?)?;

        // Load in the root template.
        // This template defines what the actual UI will look like, it contains components in the
        // layout you want them to be in, and with the attributes you want them to have.
        let root_template = Template::from_reader(ctx.filesystem.open("/mark/ui.mark")?)?;

        // Finally, actually set up the UI itself.
        let (ui, ui_root) = Ui::new(
            &root_template, None, style,
            screen_size,
            &ui_context,
        ).map_err(emtg)?;

        Ok(MainState {
            _ui_context: ui_context,
            ui,

            ui_input,
            ui_cache,
            ui_root,
        })
    }
}

impl EventHandler for MainState {
    fn update(&mut self, _ctx: &mut Context) -> GameResult<()> {
        while let Some(_event) = self.ui_root.event_sink().next() {
        }

        Ok(())
    }

    fn draw(&mut self, ctx: &mut Context) -> GameResult<()> {
        graphics::set_background_color(ctx, (0, 0, 0).into());
        graphics::clear(ctx);

        // Draw the UI
        {
            let mut renderer = GgezRenderer::new(ctx, &mut self.ui_cache);
            markedly::render::render(&mut renderer, &mut self.ui).map_err(emtg)?;
        }

        graphics::present(ctx);
        Ok(())
    }

    fn mouse_button_down_event(
        &mut self, _ctx: &mut Context,
        _button: MouseButton, x: i32, y: i32
    ) {
        self.ui_input.handle_drag_started(Point2::new(x as f32, y as f32), &mut self.ui);
    }

    fn mouse_button_up_event(
        &mut self, _ctx: &mut Context,
        _button: MouseButton, x: i32, y: i32
    ) {
        self.ui_input.handle_drag_ended(Point2::new(x as f32, y as f32), &mut self.ui);
    }

    fn mouse_motion_event(
        &mut self, _ctx: &mut Context,
        _state: MouseState, x: i32, y: i32, _xrel: i32, _yrel: i32
    ) {
        self.ui_input.handle_cursor_moved(Point2::new(x as f32, y as f32), &mut self.ui);
    }
}
