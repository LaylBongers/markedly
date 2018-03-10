extern crate ggez;
extern crate markedly;
extern crate markedly_ggez;

use std::env;
use std::path;

use ggez::{Context, GameResult, GameError};
use ggez::conf::{Conf, WindowMode, WindowSetup};
use ggez::event::{self, EventHandler, MouseButton, MouseState};
use ggez::graphics::{self, Font, Point2, Vector2};

use markedly::class::{ComponentClasses};
use markedly::input::{Input};
use markedly::scripting::{ScriptRuntime};
use markedly::template::{Template, Style};
use markedly::{Context as UiContext, Ui, Tree};

use markedly_ggez::{GgezRenderer, GgezComponentCache, emtg};

fn main() {
    // Set up the ggez context
    let mut c = Conf::new();
    c.window_mode = WindowMode {
        width: 1280,
        height: 720,
        .. Default::default()
    };
    c.window_setup = WindowSetup {
        title: "Space Game".into(),
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
    ui_font: Font,
    ui_cache: GgezComponentCache,
    ui_root: Tree,
}

impl MainState {
    pub fn new(ctx: &mut Context) -> GameResult<Self> {
        let screen_size = Vector2::new(1280.0, 720.0);

        // Set up everything needed for the UI
        let mut classes = ComponentClasses::new();
        classes.register::<markedly::class::ContainerClass>("container");
        classes.register::<markedly::class::ButtonClass>("button");

        let ui_context = UiContext {
            classes,
            runtime: ScriptRuntime::new(),
        };
        let ui_input = Input::new();
        let ui_font = Font::new(ctx, "/Raleway-Regular.ttf", 12)?;
        let ui_cache = GgezComponentCache::new();

        // Set up the UI itself
        let style = Style::from_reader(ctx.filesystem.open("/mark/_style.mark")?)?;
        let root_template = Template::from_reader(ctx.filesystem.open("/mark/ui.mark")?)?;
        let (ui, ui_root) = Ui::new(
            &root_template, None, style,
            screen_size,
            &ui_context,
        ).map_err(emtg)?;

        Ok(MainState {
            _ui_context: ui_context,
            ui,

            ui_input,
            ui_font,
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
            let mut renderer = GgezRenderer::new(ctx, &mut self.ui_cache, &self.ui_font);
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
