use clap::Parser;
use std::fs;
use std::io::Write;
use std::process::{Command, Stdio};
use scalar_lang::Lexer;
use scalar_bridge::Bridge;
use ferrous_engine::Renderer;

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
}

fn main() -> anyhow::Result<()> {
    let args = Args::parse();

    // 1. Load and parse script
    let content = fs::read_to_string(&args.input)?;
    
    // Tokenize
    use logos::Logos;
    let tokens: Vec<_> = scalar_lang::lexer::Token::lexer(&content)
        .spanned()
        .filter_map(|(t, s)| t.ok().map(|token| (token, s)))
        .collect();

    // Parse
    let ast = scalar_lang::parse(tokens).map_err(|e| anyhow::anyhow!("Parse error: {:?}", e))?;

    // 2. Initialize Headless Renderer
    let renderer = Renderer::builder()
        .with_dimensions(1920, 1080)
        .with_headless_mode(true)
        .with_fps(args.fps)
        .build()
        .map_err(|e| anyhow::anyhow!("Failed to init renderer: {}", e))?;

    // 3. Setup Bridge and Environment
    let bridge = Bridge::new(renderer);
    let mut env = scalar_lang::Environment::new();
    bridge.register_functions(&mut env);

    // 4. Run script once to populate timeline
    {
        let mut r = bridge.renderer.borrow_mut();
        // Set a better default camera
        let camera = r.camera_mut();
        camera.look_at(
            ferrous_engine::glam::Vec3::new(0.0, 10.0, 25.0),
            ferrous_engine::glam::Vec3::new(0.0, 2.0, 0.0),
        );
    }

    println!("Executing script: {}", args.input);
    scalar_lang::evaluate(ast, &mut env).map_err(|e| anyhow::anyhow!("Eval error: {}", e))?;

    // 5. Prepare FFmpeg process
    let mut ffmpeg = Command::new("ffmpeg")
        .args([
            "-y",
            "-f", "rawvideo",
            "-vcodec", "rawvideo",
            "-s", "1920x1080",
            "-pix_fmt", "rgba",
            "-r", &args.fps.to_string(),
            "-i", "-",
            "-c:v", "libx264",
            "-pix_fmt", "yuv420p",
            &args.output,
        ])
        .stdin(Stdio::piped())
        .spawn()?;

    let mut stdin = ffmpeg.stdin.take().expect("Failed to open ffmpeg stdin");

    // 6. Deterministic Render Loop
    let total_frames = (args.duration * args.fps as f64) as u64;
    println!("Rendering {} frames...", total_frames);

    for frame_idx in 0..total_frames {
        let time = frame_idx as f64 / args.fps as f64;
        
        let mut r = bridge.renderer.borrow_mut();
        if let Some(pixels) = r.render_frame(time) {
            stdin.write_all(&pixels)?;
        }

        if frame_idx % 10 == 0 {
            println!("Progress: {}/{}", frame_idx, total_frames);
        }
    }

    drop(stdin);
    ffmpeg.wait()?;
    println!("Video saved to {}", args.output);

    Ok(())
}
