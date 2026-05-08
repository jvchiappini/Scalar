use clap::Parser;
use std::fs;
use std::io::Write;
use std::process::{Command, Stdio};
use std::path::{Path, PathBuf};
use scalar_bridge::Bridge;
use ferrous_engine::{App, AppContext, FerrousApp, Renderer};
use notify::{Watcher, RecursiveMode, Event};
use std::sync::mpsc::{channel, Receiver};
use ariadne::{Color as AriadneColor, Label, Report, ReportKind, Source};
use chumsky::error::Simple;
use logos::Logos;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    #[arg(short, long)]
    input: String,

    #[arg(short, long, default_value = "output.mp4")]
    output: String,

    #[arg(short, long, default_value_t = 60)]
    fps: u32,

    #[arg(short, long, default_value_t = 5.0)]
    duration: f64,

    #[arg(short, long, default_value_t = false)]
    preview: bool,
}

fn find_std_scl() -> Option<PathBuf> {
    let current_dir = std::env::current_dir().ok()?;
    let p1 = current_dir.join("std.scl");
    if p1.exists() { return Some(p1); }
    
    let exec_dir = std::env::current_exe().ok()?.parent()?.to_path_buf();
    let p2 = exec_dir.join("std.scl");
    if p2.exists() { return Some(p2); }
    
    None
}

fn report_error(src: &str, filename: &str, errors: Vec<Simple<scalar_lang::lexer::Token>>) {
    for error in errors {
        let span = error.span();
        Report::build(ReportKind::Error, filename, span.start)
            .with_message(format!("{:?}", error))
            .with_label(
                Label::new((filename, span))
                    .with_message("Found here")
                    .with_color(AriadneColor::Red),
            )
            .finish()
            .eprint((filename, Source::from(src)))
            .unwrap();
    }
}

struct ScalarPreviewApp {
    input_path: String,
    rx: Receiver<notify::Result<Event>>,
    needs_reload: bool,
    start_time: std::time::Instant,
    headless_bridge: Option<Bridge>,
}

impl FerrousApp for ScalarPreviewApp {
    fn setup(&mut self, ctx: &mut AppContext<'_>) {
        println!("Live Preview active for: {}", self.input_path);
        
        // Initialize persistent headless bridge exactly once to avoid WGPU Device leaks
        let renderer = Renderer::builder()
            .with_dimensions(ctx.width(), ctx.height())
            .with_headless_mode(true)
            .build()
            .unwrap();
            
        self.headless_bridge = Some(Bridge::new(renderer));
        
        self.reload(ctx);
    }

    fn update(&mut self, ctx: &mut AppContext<'_>) {
        while let Ok(Ok(event)) = self.rx.try_recv() {
            if event.kind.is_modify() {
                self.needs_reload = true;
            }
        }

        if self.needs_reload {
            self.reload(ctx);
            self.start_time = std::time::Instant::now();
            self.needs_reload = false;
        }

        // Advance time for animations!
        let t = self.start_time.elapsed().as_secs_f64();
        let mut resources = ferrous_ecs::prelude::ResourceMap::new();
        resources.insert(ferrous_core::Time {
            delta: 0.0,
            elapsed: t,
            frame_count: 0,
            fps: 0.0,
        });

        // Run the AnimatorSystem over the live world so the timeline plays
        let mut animator = ferrous_core::scene::AnimatorSystem;
        ferrous_ecs::system::System::run(&mut animator, &mut ctx.world.ecs, &mut resources);
    }
}

impl ScalarPreviewApp {
    fn reload(&mut self, ctx: &mut AppContext<'_>) {
        let content = match fs::read_to_string(&self.input_path) {
            Ok(c) => c,
            Err(e) => {
                eprintln!("Failed to read {}: {}", self.input_path, e);
                return;
            }
        };

        let tokens: Vec<_> = scalar_lang::lexer::Token::lexer(&content).spanned()
            .filter_map(|(t, s)| t.ok().map(|token| (token, s))).collect();

        let ast = match scalar_lang::parse(tokens) {
            Ok(ast) => ast,
            Err(e) => {
                report_error(&content, &self.input_path, e);
                return;
            }
        };

        // Free old materials before clearing App scene
        let mut to_free = Vec::new();
        for (_e, mc) in ctx.world.ecs.query::<ferrous_core::scene::MaterialComponent>() {
            if mc.handle.0 != 0 {
                to_free.push(mc.handle);
            }
        }
        let r = ctx.render.renderer_mut();
        for handle in to_free {
            r.material_registry.free(handle);
        }

        // Clear App scene
        ctx.world.clear();

        // Evaluate the AST using the persistent headless bridge
        let bridge = self.headless_bridge.as_ref().unwrap();
        bridge.renderer.borrow_mut().clear(); // Clear headless scene

        let mut env = scalar_lang::Environment::new();
        bridge.register_functions(&mut env);

        if let Some(std_path) = find_std_scl() {
            if let Ok(std_content) = fs::read_to_string(std_path) {
                let std_tokens: Vec<_> = scalar_lang::lexer::Token::lexer(&std_content).spanned()
                    .filter_map(|(t, s)| t.ok().map(|token| (token, s))).collect();
                if let Ok(std_ast) = scalar_lang::parse(std_tokens) {
                    let _ = scalar_lang::evaluate(std_ast, &mut env);
                }
            }
        }

        if let Err(e) = scalar_lang::evaluate(ast, &mut env) {
             eprintln!("Eval Error: {}", e);
        }

        // Final sync: Copy results from bridge to App
        let b = bridge.renderer.borrow_mut();
        
        ctx.render.renderer_mut().camera_system.camera = b.gpu.camera_system.camera.clone();
        ctx.world.ecs = b.world.ecs.clone();

        // Fix cross-device MaterialHandles: 
        // 1. App's renderer doesn't have the materials created by the headless renderer.
        // 2. We must allocate new bind groups natively in the App's WGPU device.
        let r = ctx.render.renderer_mut();
        r.world_material_descs.clear();
        r.shape_batcher.clear();

        let mut old_to_new = std::collections::HashMap::new();

        let entities: Vec<_> = ctx.world.ecs.query::<ferrous_core::scene::MaterialComponent>().map(|(e, _)| e).collect();
        for entity in entities {
            let mut desc_clone = None;
            if let Some(mut mc) = ctx.world.ecs.get_mut::<ferrous_core::scene::MaterialComponent>(entity) {
                let old_handle = mc.handle.0;
                if old_handle != 0 {
                    let new_handle = if let Some(&h) = old_to_new.get(&old_handle) {
                        h
                    } else {
                        let h = r.material_registry.create(&r.context.device, &r.context.queue, &mc.descriptor);
                        old_to_new.insert(old_handle, h);
                        h
                    };
                    mc.handle = new_handle;
                }
                desc_clone = Some(mc.descriptor.clone());
            }

            if let Some(d) = desc_clone {
                r.world_material_descs.insert(entity.to_bits(), d);
            }
        }
        
        println!("Hot-reload complete. Timeline restarted.");
    }
}

fn main() -> anyhow::Result<()> {
    let args = Args::parse();

    if args.preview {
        run_preview(args)
    } else {
        run_headless(args)
    }
}

fn run_headless(args: Args) -> anyhow::Result<()> {
    let content = fs::read_to_string(&args.input)?;
    let tokens: Vec<_> = scalar_lang::lexer::Token::lexer(&content).spanned()
        .filter_map(|(t, s)| t.ok().map(|token| (token, s))).collect();
    let ast = scalar_lang::parse(tokens).map_err(|e| anyhow::anyhow!("Parse error: {:?}", e))?;

    let renderer = Renderer::builder()
        .with_dimensions(1920, 1080)
        .with_headless_mode(true)
        .with_fps(args.fps)
        .build()?;

    let bridge = Bridge::new(renderer);
    let mut env = scalar_lang::Environment::new();
    bridge.register_functions(&mut env);

    if let Some(std_path) = find_std_scl() {
        if let Ok(std_content) = fs::read_to_string(std_path) {
            let std_tokens: Vec<_> = scalar_lang::lexer::Token::lexer(&std_content).spanned()
                .filter_map(|(t, s)| t.ok().map(|token| (token, s))).collect();
            if let Ok(std_ast) = scalar_lang::parse(std_tokens) {
                let _ = scalar_lang::evaluate(std_ast, &mut env);
            }
        }
    }

    scalar_lang::evaluate(ast, &mut env).map_err(|e| anyhow::anyhow!("Eval error: {}", e))?;

    let mut ffmpeg = Command::new("ffmpeg")
        .args([
            "-y", "-f", "rawvideo", "-vcodec", "rawvideo", "-s", "1920x1080",
            "-pix_fmt", "rgba", "-r", &args.fps.to_string(), "-i", "-",
            "-c:v", "libx264", "-pix_fmt", "yuv420p", &args.output,
        ])
        .stdin(Stdio::piped())
        .spawn()?;

    let mut stdin = ffmpeg.stdin.take().expect("Failed to open ffmpeg stdin");
    let total_frames = (args.duration * args.fps as f64) as u64;
    for frame_idx in 0..total_frames {
        let time = frame_idx as f64 / args.fps as f64;
        let mut r = bridge.renderer.borrow_mut();
        if let Some(pixels) = r.render_frame(time) {
            stdin.write_all(&pixels)?;
        }
    }
    drop(stdin);
    ffmpeg.wait()?;
    Ok(())
}

fn run_preview(args: Args) -> anyhow::Result<()> {
    let (tx, rx) = channel();
    let mut watcher = notify::recommended_watcher(tx)?;
    watcher.watch(Path::new(&args.input), RecursiveMode::NonRecursive)?;

    let app = ScalarPreviewApp {
        input_path: args.input,
        rx,
        needs_reload: false,
        start_time: std::time::Instant::now(),
        headless_bridge: None,
    };

    App::new(app)
        .with_title("Scalar Live Preview")
        .with_size(1280, 720)
        .run();

    Ok(())
}
