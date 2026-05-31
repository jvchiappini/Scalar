# Análisis: Problema de Renderizado 2D en Scalar + FerrousEngine

## Resumen

El renderizador 2D no funciona correctamente porque el **bridge de Scalar mantiene el `RendererMode::Full3D`** en lugar de cambiar a `Pure2D`. Las formas 2D se crean como entidades ECS que son teseladas por lyon y renderizadas a través del pipeline WorldPass 3D completo (prepass, SSAO, depth testing, MSAA, post-process). **core_viewer** funciona porque usa `enable_pure_2d()` + `ShapeInstance` (modo inmediato SDF), que salta todo el pipeline 3D.

---

## Arquitectura Actual

### Flujo de Scalar (NO funciona correctamente):

1. `test_demo.scl` → `Camera.set_mode_2d(true)`
2. Bridge (`camera.rs:65-78`) llama a `camera.set_mode_2d(true, Some(height))` 
3. Esto SOLO cambia la cámara a **orthográfica** (eye=(0,0,10), proyección ortho)
4. El **RendererMode sigue siendo `Full3D`** (nunca se cambia)
5. Las formas se crean como entidades ECS vía `spawn_2d_circle/spawn_2d_line/etc.`
6. Se renderizan por WorldPass con teselación lyon → geometría 3D → pipeline PBR/FlatShaded
7. Pipeline completo ejecutado: Prepass → SSAO → WorldPass → Gizmos → Technical 2D → AA → Post-process

### Flujo de core_viewer (SÍ funciona):

1. `enable_pure_2d(background_color)` → cambia a `RendererMode::Pure2D`
2. Configura cámara ortho pixel-perfect (1 unidad = 1 pixel)
3. Cada frame: `clear_2d()` + `draw_2d_shape(ShapeInstance::line/circle/etc.)`
4. Pipeline mínimo: Clear → Shapes SDF → Sprites → UI
5. Sin prepass, SSAO, WorldPass, depth buffer, ni post-process

---

## Causa Raíz

**El bridge de Scalar nunca llama a `renderer.enable_pure_2d()`**, solo a `camera.set_mode_2d(true)`. Esto mantiene el renderer en `Full3D`, lo que causa:

1. **Las formas 2D son teseladas como mallas 3D** por lyon en vez de renderizadas como quads SDF
2. **Pasa por prepass + SSAO** que puede obscurecer o artefactar las formas planas
3. **Depth testing** activo (CompareFunction::LessEqual) que puede causar problemas de ordenamiento
4. **MSAA** (4x) añadido a formas 2D que no lo necesitan
5. **Post-process** (tone mapping) puede alterar colores
6. **El shape_batcher está vacío** en el Technical 2D Pass, porque las formas son ECS, no ShapeInstance

---

## Problemas Específicos Identificados

### 1. RendererMode incorrecto
- **Archivo**: `scalar_bridge/src/bindings/camera.rs:65-78`
- **Problema**: `Camera.set_mode_2d(true)` solo cambia la cámara, no el modo del renderizador
- **Fix**: Llamar `renderer.enable_pure_2d(background)` en vez de solo `camera.set_mode_2d()`

### 2. ECS entities no se sincronizan al shape_batcher
- **Archivo**: `ferrous_engine/src/renderer.rs:199-284`
- **Problema**: `spawn_2d_circle/spawn_2d_line/spawn_2d_rect` crean entidades ECS que solo se renderizan en WorldPass (Full3D). En Pure2D, el shape_batcher está vacío.
- **Fix**: Añadir método que convierta entidades ECS 2D a ShapeInstance y las pushee al shape_batcher

### 3. Background color se setea incorrectamente
- **Archivo**: `scalar_bridge/src/bindings/camera.rs:74-76`
- **Problema**: `Camera.set_mode_2d(true)` setea background a BLACK, pero luego `set_background_color()` lo sobrescribe
- **Fix**: Integrar el color de fondo en `enable_pure_2d()`

### 4. Aspect ratio mal calculado en set_mode_2d
- **Archivo**: `ferrous_core/src/scene/camera.rs:125-148`
- **Problema**: `set_mode_2d()` lee el aspect_ratio de la proyección actual (Perspective) ANTES de cambiarla. Si `set_aspect()` no se ha llamado antes, el aspect_ratio es 1.0 (default).
- **Fix**: `enable_pure_2d()` ya llama `set_aspect()` después de `set_mode_2d()` para corregir esto

---

## Solución Propuesta

### Cambio 1: `ferrous_engine/src/renderer.rs` - Añadir sync ECS→batcher

Añadir método que itere entidades 2D en el ECS world y las convierta a `ShapeInstance`:

```rust
pub fn sync_2d_to_shape_batcher(&mut self) {
    use ferrous_2d::render::types::ShapeInstance;
    use ferrous_core::scene::ElementKind;
    
    self.gpu.shape_batcher.clear();
    
    for (entity, (element, transform)) in self.world.ecs.query::<(&Element, &Transform)>() {
        if !element.visible { continue; }
        let pos = transform.position;
        let color = element.fill_color.unwrap_or(element.stroke_color.unwrap_or([1.0, 1.0, 1.0, 1.0]));
        
        match &element.kind {
            ElementKind::Path => {
                if let Some(path_data) = self.world.ecs.get::<PathData>(entity) {
                    // Convert path to line segments and push to batcher
                    self.path_to_shape_instances(path_data, pos, color);
                }
            }
            ElementKind::Circle2D { radius, .. } => {
                let instance = ShapeInstance::circle_z(
                    glam::Vec2::new(pos.x, pos.y), pos.z, *radius, color
                );
                self.gpu.draw_2d_shape(instance);
            }
            ElementKind::Rect2D { width, height } => {
                let instance = ShapeInstance::rect_z(
                    glam::Vec2::new(pos.x, pos.y), pos.z,
                    glam::Vec2::new(*width, *height), color
                );
                self.gpu.draw_2d_shape(instance);
            }
            ElementKind::Line2D { x0, y0, x1, y1, thickness } => {
                let stroke_color = element.stroke_color.unwrap_or([1.0, 1.0, 1.0, 1.0]);
                // Aplicar transform de posición
                let from = glam::Vec2::new(pos.x + x0, pos.y + y0);
                let to = glam::Vec2::new(pos.x + x1, pos.y + y1);
                let instance = ShapeInstance::line(from, to, *thickness, stroke_color);
                self.gpu.draw_2d_shape(instance);
            }
            _ => {}
        }
    }
}
```

### Cambio 2: Modificar `render_frame()` para llamar sync antes de renderizar

```rust
pub fn render_frame(&mut self, t: f64) -> Option<Vec<u8>> {
    // ... animator system ...
    self.animator_system.run(&mut self.world.ecs, &mut resources);

    // Sync ECS world to GPU
    self.gpu.sync_world(&self.world);
    
    // Sync ECS 2D entities to shape_batcher (para Pure2D mode)
    self.sync_2d_to_shape_batcher();

    // ... begin encoding, render, readback ...
}
```

### Cambio 3: `scalar_bridge/src/bindings/camera.rs` - Usar enable_pure_2d

```rust
camera_obj.insert("set_mode_2d".to_string(), Value::NativeFunction(Rc::new(move |args, _| {
    let enabled = match args.get(1) {
        Some(Value::Boolean(b)) => *b,
        _ => true,
    };
    let mut ren = r.borrow_mut();
    if enabled {
        let bg = wgpu::Color {
            r: 0.05, g: 0.05, b: 0.12, a: 1.0,
        };
        ren.gpu.enable_pure_2d(bg);
    }
    Ok(Value::Number(0.0))
})));
```

### Cambio 4: Ajustar `set_background_color` para Pure2D

El `set_background_color` del script debe actualizar el clear_color en Pure2D igual que en Full3D.

---

## Archivos Clave Modificados

| Archivo | Cambio |
|---------|--------|
| `ferrous_engine/src/renderer.rs` | Añadir `sync_2d_to_shape_batcher()`, modificar `render_frame()` |
| `scalar_bridge/src/bindings/camera.rs` | Llamar `enable_pure_2d()` |
| `scalar_bridge/src/bindings/shapes.rs` | Ajustar set_fill/set_stroke para Pure2D (opcional) |

## Archivos de Referencia (core_viewer, funciona correctamente)

| Archivo | Propósito |
|---------|-----------|
| `core_viewer/src/lib.rs` | `enable_pure_2d()`, `clear_2d()`, setup correcto |
| `core_viewer/src/editor/tools/wall/renderer_2d.rs` | Uso de `ShapeInstance::line/circle` |
| `ferrous_2d/src/render/types.rs` | Constructores `ShapeInstance` |
