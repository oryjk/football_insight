"""Build a local inference helper without changing shared Cargo registry sources.

Run with uv run python scripts/setup_qwen_native.py. Model weights are not downloaded.
"""
from __future__ import annotations

import argparse
import hashlib
import shutil
import subprocess
import sys
import tarfile
import tempfile
from pathlib import Path
from urllib.request import urlopen

CRATE_SHA256 = 'a0db17af7a58bf8baf83d00e3d37cb862ba952bc623ca178b7706fee8df64fe3'


def main() -> None:
    parser = argparse.ArgumentParser()
    parser.add_argument('--device', choices=['metal', 'cuda', 'cpu'], default='metal' if sys.platform == 'darwin' else 'cpu')
    parser.add_argument('--target-dir', type=Path, help='optional existing Cargo build cache')
    args = parser.parse_args()
    root = Path(__file__).resolve().parents[1]
    template = root / 'tools/qwen-native'
    build = root / 'data/native-qwen'
    build.mkdir(parents=True, exist_ok=True)
    for name in ['Cargo.toml', 'Cargo.lock']:
        if name == 'Cargo.lock' and (build / name).exists():
            continue
        shutil.copyfile(template / name, build / name)
    shutil.copytree(template / 'src', build / 'src', dirs_exist_ok=True)
    vendor = build / 'vendor/any-tts'
    marker = vendor / '.football-patch-v1'
    if not marker.exists():
        cached = list((Path.home() / '.cargo/registry/cache').glob('*/any-tts-0.2.0.crate'))
        if cached:
            content = cached[0].read_bytes()
        else:
            with urlopen('https://static.crates.io/crates/any-tts/any-tts-0.2.0.crate', timeout=60) as response:
                content = response.read()
        if hashlib.sha256(content).hexdigest() != CRATE_SHA256:
            raise RuntimeError('any-tts archive hash mismatch')
        with tempfile.TemporaryDirectory() as directory:
            archive = Path(directory) / 'any-tts.crate'
            archive.write_bytes(content)
            with tarfile.open(archive) as tar:
                tar.extractall(directory, filter='data')
            if vendor.exists():
                shutil.rmtree(vendor)
            shutil.copytree(Path(directory) / 'any-tts-0.2.0', vendor)
        subprocess.run(['patch', '-p1', '-i', str(template / 'any-tts-0.6b.patch')], cwd=vendor, check=True)
        marker.write_text('0.2.0 + 0.6B text projection / optional code projection\n')
    command = ['cargo', 'build', '--release', '--locked', '--manifest-path', str(build / 'Cargo.toml')]
    if args.device != 'cpu':
        command += ['--features', args.device]
    if args.target_dir:
        command += ['--target-dir', str(args.target_dir.resolve())]
    subprocess.run(command, check=True)
    binary = (args.target_dir.resolve() if args.target_dir else build / 'target') / 'release/football-qwen-tts'
    install = build / 'bin/football-qwen-tts'
    install.parent.mkdir(exist_ok=True)
    shutil.copy2(binary, install)
    print(f'FI_QWEN_RUNNER={install}')


if __name__ == '__main__':
    main()
