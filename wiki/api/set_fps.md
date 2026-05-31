# SetFPS

**Syntax:** `SetFPS(fps)`

**Description:**
Overrides the output frame rate of the video. The default comes from the CLI `--fps` argument; calling `SetFPS()` in the script takes priority.

## Argument

| Argument | Type | Obligatorio | Description |
|----------|------|:-----------:|-------------|
| `fps` | Number | ✓ | Target frames per second (`>= 1`) |

## Example

```scalar
SetFPS(60)   // 60 fps output
SetFPS(30)   // 30 fps (slower playback, less processing)
SetFPS(120)  // 120 fps (smoother, more frames)
```
