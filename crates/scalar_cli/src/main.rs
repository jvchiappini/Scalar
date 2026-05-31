use clap::Parser;
use std::fs;
use std::io::Write;
use std::process::{Command, Stdio};
use std::path::PathBuf;
use std::sync::mpsc;
use std::thread;
use scalar_bridge::Bridge;
use logos::Logos;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Input .scl script file
    #[arg(short, long)]
    input: String,

    /// Output video file (mp4)
    #[arg(short, long, default_value = "output.mp4")]
    output: String,

    /// Frames per second (default, overridable via SetFPS() in script)
    #[arg(short, long, default_value_t = 60)]
    fps: u32,

    /// Duration in seconds
    #[arg(short, long, default_value_t = 5.0)]
    duration: f64,

    /// Render width (default 1920)
    #[arg(long, default_value_t = 1920)]
    width: u32,

    /// Render height (default 1080)
    #[arg(long, default_value_t = 1080)]
    height: u32,
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

fn main() -> anyhow::Result<()> {
    let args = Args::parse();

    // ── Parse the script ──────────────────────────────────────────
    let content = fs::read_to_string(&args.input)?;
    let tokens: Vec<_> = scalar_lang::lexer::Token::lexer(&content).spanned()
        .filter_map(|(t, s)| t.ok().map(|token| (token, s))).collect();
    let ast = scalar_lang::parse(tokens)
        .map_err(|e| anyhow::anyhow!("Parse error: {:?}", e))?;

    // ── Initialise renderer (Pure2D headless) ─────────────────────
    let bridge = Bridge::new(args.width, args.height, args.fps)?;
    let mut env = scalar_lang::Environment::new();
    bridge.register_functions(&mut env);

    // ── Evaluate std library (if present) ────────────────────────
    if let Some(std_path) = find_std_scl() {
        if let Ok(std_content) = fs::read_to_string(std_path) {
            let std_tokens: Vec<_> = scalar_lang::lexer::Token::lexer(&std_content).spanned()
                .filter_map(|(t, s)| t.ok().map(|token| (token, s))).collect();
            if let Ok(std_ast) = scalar_lang::parse(std_tokens) {
                let _ = scalar_lang::evaluate(std_ast, &mut env);
            }
        }
    }

    // ── Run user script ───────────────────────────────────────────
    scalar_lang::evaluate(ast, &mut env)
        .map_err(|e| anyhow::anyhow!("Eval error: {}", e))?;

    // ── Read settings set by script ───────────────────────────────
    let (w, h) = {
        let r = bridge.renderer.borrow();
        (r.width(), r.height())
    };
    let output_fps = *bridge.fps.borrow();
    let motion_blur = *bridge.motion_blur_samples.borrow();
    let sub_samples = if motion_blur > 0 { motion_blur } else { 1 };

    let output_frame_dt = 1.0 / output_fps as f64;
    let sub_frame_dt = output_frame_dt / sub_samples as f64;
    let total_output_frames = (args.duration * output_fps as f64) as u64;

    eprintln!("  Resolution: {}×{}", w, h);
    eprintln!("  FPS: {} output × {} sub-samples = {} renders/sec",
        output_fps, sub_samples, output_fps * sub_samples);
    eprintln!("  Duration: {}s → {} frames",
        args.duration, total_output_frames);

    // ── Launch ffmpeg ────────────────────────────────────────────
    let ffmpeg_args: Vec<String> = vec![
        "-y".into(), 
        // Read uncompressed raw video fast
        "-hwaccel".into(), "auto".into(),
        "-f".into(), "rawvideo".into(), "-vcodec".into(), "rawvideo".into(),
        "-s".into(), format!("{}x{}", w, h),
        "-pix_fmt".into(), "rgba".into(),
        "-r".into(), output_fps.to_string(),
        "-i".into(), "-".into(),
        // Hardware encoder if possible, or fallback to fast x264 with minimal memory footprint
        "-c:v".into(), "libx264".into(), 
        "-preset".into(), "ultrafast".into(), // crucial for 8GB RAM machines at 4K
        "-threads".into(), "4".into(),        // limit x264 memory overhead per thread
        "-pix_fmt".into(), "yuv420p".into(),
        args.output.clone(),
    ];

    let mut ffmpeg = Command::new("ffmpeg")
        .args(&ffmpeg_args)
        .stdin(Stdio::piped())
        .spawn()?;

    let mut stdin = ffmpeg.stdin.take().expect("Failed to open ffmpeg stdin");

    // ── Buffer pool ──────────────────────────────────────────────
    // All buffers are pre-allocated and recycled to avoid malloc/free churn.
    let buf_size = (w * h * 4) as usize;

    // Scratch buffer for motion blur sub-frames (reused)
    let mut scratch_buf: Vec<u8> = Vec::with_capacity(buf_size);

    // Accumulation buffer (u32 to avoid overflow when summing)
    let mut accum: Vec<u32> = Vec::with_capacity(buf_size);

    // Pool of output buffers shared between render & writer threads
    let (write_tx, write_rx) = mpsc::sync_channel::<Vec<u8>>(4);

    // Writer thread: reads from channel, writes to ffmpeg, returns buffer to pool
    let (return_tx, return_rx) = mpsc::channel::<Vec<u8>>();
    let return_tx_thread = return_tx.clone();
    let writer_handle = thread::spawn(move || {
        for mut buf in write_rx {
            if stdin.write_all(&buf).is_err() { break; }
            buf.clear();
            // Return buffer to pool for reuse
            let _ = return_tx_thread.send(buf);
        }
    });

    // Pre-fill the pool with some empty buffers
    for _ in 0..4 {
        let _ = return_tx.send(Vec::with_capacity(buf_size));
    }

    // ── Render loop ──────────────────────────────────────────────
    let start_time = std::time::Instant::now();
    let mut total_renders: u64 = 0;

    for frame_idx in 0..total_output_frames {
        if motion_blur == 0 {
            // ── No motion blur: one render per output frame ─────
            let time = frame_idx as f64 / output_fps as f64;
            bridge.before_frame(time);

            // Get a fresh buffer from pool
            let mut out = return_rx.recv().unwrap_or_else(|_| Vec::with_capacity(buf_size));
            out.clear();

            let rendered = bridge.renderer.borrow_mut().render_frame_into(time, &mut out);
            if rendered {
                if write_tx.send(out).is_err() { break; }
            }
            total_renders += 1;
        } else {
            // ── Motion blur: render sub-frames and average ──────
            accum.clear();

            for sample in 0..sub_samples {
                let sub_idx = frame_idx * sub_samples as u64 + sample as u64;
                let time = sub_idx as f64 * sub_frame_dt;

                bridge.before_frame(time);

                scratch_buf.clear();
                let rendered = bridge.renderer.borrow_mut().render_frame_into(time, &mut scratch_buf);

                if !rendered { continue; }

                if sample == 0 {
                    accum.extend(scratch_buf.iter().map(|&p| p as u32));
                } else {
                    for (a, &p) in accum.iter_mut().zip(scratch_buf.iter()) {
                        *a += p as u32;
                    }
                }
                total_renders += 1;
            }

            // Get output buffer from pool
            let mut out = return_rx.recv().unwrap_or_else(|_| Vec::with_capacity(buf_size));
            out.clear();

            let div = sub_samples as u32;
            out.extend(accum.iter().map(|&v| (v / div) as u8));

            if write_tx.send(out).is_err() { break; }
        }

        // Progress indicator every second
        if frame_idx % (output_fps as u64) == 0 && frame_idx > 0 {
            let elapsed = start_time.elapsed();
            let pct = frame_idx as f64 / total_output_frames as f64 * 100.0;
            let total_est = elapsed / frame_idx as u32 * total_output_frames as u32;
            eprintln!("  {:3.0}% — {} renders in {:.1}s (est {:.1}s)",
                pct, total_renders, elapsed.as_secs_f64(), total_est.as_secs_f64());
        }
    }

    let elapsed = start_time.elapsed();
    eprintln!("  100% — {} renders in {:.1}s ({:.0} renders/sec, {:.1} MPixels/sec)",
        total_renders, elapsed.as_secs_f64(),
        total_renders as f64 / elapsed.as_secs_f64(),
        total_renders as f64 * (w * h) as f64 / elapsed.as_secs_f64() / 1_000_000.0);

    drop(write_tx);
    writer_handle.join().unwrap();
    ffmpeg.wait()?;
    println!("Done → {}", args.output);
    Ok(())
}
