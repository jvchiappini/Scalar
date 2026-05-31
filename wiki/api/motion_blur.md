# MotionBlur

**Syntax:** `MotionBlur(samples)`

**Description:**
Enables or disables motion blur by rendering multiple sub-frames per output frame and averaging them. Each sub-frame captures the scene at a slightly different point in time, producing smooth motion trails for animated elements.

## Argument

| Argument | Type | Obligatorio | Description |
|----------|------|:-----------:|-------------|
| `samples` | Number | ✓ | Number of sub-samples per frame (`0` = off, `4` = recommended) |

## How it works

When `samples > 0`, the engine renders `samples` sub-frames for each output frame at evenly-spaced time intervals within the frame period, then averages their pixel buffers. This creates a realistic motion blur effect on moving/animated elements.

The total render cost scales linearly: `samples × output_frames` instead of `output_frames`.

## Example

```scalar
MotionBlur(0)    // Off (default)
MotionBlur(4)    // 4 sub-samples — good quality
MotionBlur(8)    // 8 sub-samples — higher quality, 2x render time
MotionBlur(16)   // 16 sub-samples — very smooth, 4x render time
```

## Performance

| Samples | Render cost | Quality |
|---------|-------------|---------|
| 0 | 1× output frames | No motion blur |
| 4 | 4× output frames | Good |
| 8 | 8× output frames | High |
| 16 | 16× output frames | Very high |

## Notes

- Motion blur is most visible on fast-moving animations (e.g., `ease_out_bounce` plots).
- Each sub-frame advances the animation system incrementally, producing accurate temporal sampling.
- The averaged frames are piped directly to ffmpeg; no post-processing filter is needed.
