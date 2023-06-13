#!/usr/bin/env python
from argparse import ArgumentParser, Namespace
from typing import assert_never
import subprocess
import tomllib
from pathlib import Path, PosixPath
import functools
import shutil
import platform


# === global config ===
args: None | Namespace = None
cargo_config = tomllib.loads(Path("Cargo.toml").read_text())
bundler_config = tomllib.loads(Path("bundler.toml").read_text())
build_config = tomllib.loads(Path("build.toml").read_text())

# TODO: this is not complete!
dev_packages = {"Ubuntu": ["librust-alsa-sys-dev", "libjack-jackd2-dev"]}


def main():
    global args
    parser = ArgumentParser()
    parser.add_argument(
        "actions",
        choices=[
            "generate-filters",
            "build",
            "deploy",
            "run",
            "debug",
            "dev-install",
            "lint",
            "fix",
        ],
        nargs="+",
    )

    args = parser.parse_args()

    for a in args.actions:
        match a:
            case "dev-install":
                dev_install()
            case "generate-filters":
                generate_filters()
            case "build":
                build()
            case "deploy":
                deploy()
            case "run":
                run()
            case "debug":
                debug()
            case "lint":
                lint()
            case "fix":
                fix()
            case _:
                assert_never()


@functools.cache
def build():
    subprocess.run(
        ["cargo", "xtask", "bundle", cargo_config["package"]["name"]], check=True
    )


@functools.cache
def deploy():
    build()
    name = bundler_config[cargo_config["package"]["name"]]["name"]
    vst3_source = PosixPath(f"./target/bundled/{name}.vst3")
    clap_source = PosixPath(f"./target/bundled/{name}.clap")
    vst3_destination = PosixPath(
        f"{build_config['vst3_location']}/{name}.vst3"
    ).expanduser()
    shutil.rmtree(vst3_destination)
    clap_destination = PosixPath(
        f"{build_config['clap_location']}/{name}.clap"
    ).expanduser()
    vst3_source.replace(vst3_destination)
    print(f"Copied {name}.vst3 and {name}.clap to destination")
    clap_source.replace(clap_destination)


@functools.cache
def run():
    deploy()
    subprocess.run(build_config["run_command"], check=True)


@functools.cache
def debug():
    subprocess.run(["cargo", "run", "--features", "draw_gizmos"])


@functools.cache
def dev_install():
    os_version = platform.version().lower()
    if "ubuntu" in os_version:
        subprocess.run(["sudo", "apt-get", "update"])
        subprocess.run(
            [
                "sudo",
                "apt-get",
                "install",
            ]
            + dev_packages["Ubuntu"]
        )


@functools.cache
def lint():
    subprocess.run(["cargo", "clippy", "--", "-W", "clippy::pedantic"])


@functools.cache
def fix():
    subprocess.run(["cargo", "clippy", "--fix", "--", "-W", "clippy::pedantic"])


@functools.cache
def generate_filters():
    with open("src/dsp/filter_coefficients.rs", "w") as f:
        subprocess.run(["python3", "scripts/generate_filters.py"], stdout=f)


if __name__ == "__main__":
    main()
