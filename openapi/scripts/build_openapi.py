#!/usr/bin/env python3

from __future__ import annotations

import json
import shutil
import subprocess
from pathlib import Path

SCRIPT_DIR = Path(__file__).resolve().parent
OPENAPI_DIR = SCRIPT_DIR.parent

INPUT = OPENAPI_DIR / "spec" / "openapi.yaml"
GENERATED_DIR = OPENAPI_DIR / "generated"
OUTPUT_PRETTY = GENERATED_DIR / "openapi.json"
OUTPUT_MIN = GENERATED_DIR / "openapi.min.json"


def check_dependency(name: str) -> None:
    if shutil.which(name) is None:
        raise RuntimeError(f"'{name}' is required and was not found in PATH")


def bundle_openapi() -> None:
    GENERATED_DIR.mkdir(parents=True, exist_ok=True)

    subprocess.run(
        [
            "docker",
            "run",
            "--rm",
            "-v",
            f"{OPENAPI_DIR}:/workspace",
            "-w",
            "/workspace",
            "redocly/cli",
            "bundle",
            "spec/openapi.yaml",
            "-o",
            "generated/openapi.json",
        ],
        check=True,
    )

    if not OUTPUT_PRETTY.exists():
        raise RuntimeError(f"OpenAPI bundle was not generated: {OUTPUT_PRETTY}")


def minify_openapi() -> None:
    with OUTPUT_PRETTY.open(mode="r", encoding="utf-8") as file:
        document = json.load(file)

    with OUTPUT_MIN.open(mode="w", encoding="utf-8") as file:
        json.dump(document, file, separators=(",", ":"))

    if not OUTPUT_MIN.exists():
        raise RuntimeError(f"OpenAPI file was not generated: {OUTPUT_MIN}")


def main() -> None:
    check_dependency("docker")

    if not INPUT.exists():
        raise FileNotFoundError(f"OpenAPI specification not found: {INPUT}")

    bundle_openapi()
    minify_openapi()

    print(f"OpenAPI bundle generated: {OUTPUT_PRETTY}")
    print(f"OpenAPI bundle minified generated: {OUTPUT_MIN}")


if __name__ == "__main__":
    main()
