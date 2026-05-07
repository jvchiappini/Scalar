#!/bin/bash
set -e

# build.sh - Compila para Windows y crea un ZIP con el binario y el script de prueba.

echo "🚀 Iniciando proceso de compilación para Windows..."

# 1. Asegurar que el target de Windows esté instalado
rustup target add x86_64-pc-windows-gnu

# 2. Compilar el CLI en modo release
echo "📦 Compilando crates..."
cargo build --release --target x86_64-pc-windows-gnu -p scalar_cli

# 3. Crear estructura de distribución
echo "📁 Preparando paquete..."
DIST_DIR="dist_windows"
rm -rf "$DIST_DIR"
mkdir -p "$DIST_DIR"

# Copiar el ejecutable
cp target/x86_64-pc-windows-gnu/release/scalar_cli.exe "$DIST_DIR/"

# Copiar el script de animación de prueba
cp test_animation.scl "$DIST_DIR/"

# 4. Crear el archivo ZIP
echo "🗜️ Creando archivo ZIP..."
ZIP_NAME="scalar_bundle.zip"
rm -f "$ZIP_NAME"

# Usamos python3 para crear el ZIP si el comando zip no está disponible
python3 -c "import zipfile, os; z = zipfile.ZipFile('$ZIP_NAME', 'w'); [z.write(os.path.join('$DIST_DIR', f), f) for f in os.listdir('$DIST_DIR')]; z.close()"

echo "✅ Proceso finalizado con éxito."
echo "🎁 Archivo generado: $ZIP_NAME"
echo "Contenido del ZIP:"
unzip -l "$ZIP_NAME"
