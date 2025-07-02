from pathlib import Path
import subprocess
import re
import os

SLANGC = os.environ.get("SLANGC")

if not SLANGC: 
    raise RuntimeError("environemtn variable SLANGC not set")

INPUT_DIR = "assets/shaders/slang"
OUTPUT_DIR = "assets/shaders/spirv"

ENTRYPOINT_PATTERN = re.compile(r'\[\[shader\("(\w+)"\)\]\]\s*\w+\s+(\w+)\s*\(')

def has_entrypoint(path):
    with open(path, 'r', encoding='utf-8') as f:
        source = f.read()
    
    entrypoints = ENTRYPOINT_PATTERN.findall(source)
    return len(entrypoints) > 0

def compile(shader_path):
    # skip compilation if there is no entrypoint in the file
    if not has_entrypoint(shader_path):
        return

    relative_path = Path.relative_to(shader_path, INPUT_DIR)
    output_path = OUTPUT_DIR / relative_path.with_suffix(".spv")

    cmd = [
        SLANGC, str(shader_path),
        "-o", str(output_path),
        "-target", "spirv",
        "-O3",
        "-fvk-use-entrypoint-name"
    ]

    print(f"Compiling {shader_path} into {output_path}")
    result = subprocess.run(cmd, capture_output=True, text=True)

    if result.returncode != 0:
        print(f"Error compiling {shader_path}: {result.stderr}")
        exit(1)

def main():
    input_path = Path(INPUT_DIR)

    for shader_path in input_path.rglob("*.slang"):
        compile(shader_path)

if __name__ == "__main__":
    main()