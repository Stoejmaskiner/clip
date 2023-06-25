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
    parser.add_argument("--release", action="store_true")
    parser.add_argument("--prof", action="store_true")

    args = parser.parse_args()

    for a in args.actions:
        match a:
            case "dev-install":
                dev_install(args.release, args.prof)
            case "generate-filters":
                generate_filters(args.release, args.prof)
            case "build":
                build(args.release, args.prof)
            case "deploy":
                deploy(args.release, args.prof)
            case "run":
                run(args.release, args.prof)
            case "debug":
                debug(args.release, args.prof)
            case "lint":
                lint(args.release, args.prof)
            case "fix":
                fix(args.release, args.prof)
            case _:
                assert_never(args.actions)


@functools.cache
def build(release, prof):
    if release and prof:
        raise ValueError("can't use --release and --prof at the same time")
    cmd = ["cargo", "xtask", "bundle", cargo_config["package"]["name"]]
    if release:
        cmd.extend(["--release"])
    if prof:
        cmd.extend(["--profile", "profiling"])
    subprocess.run(cmd, check=True)


@functools.cache
def deploy(release, prof):
    build(release, prof)
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
def run(release, prof):
    deploy(release, prof)
    subprocess.run(build_config["run_command"], check=True)


@functools.cache
def debug(release, prof):
    # subprocess.run(["cargo", "run", "--features", "draw_gizmos"])
    subprocess.run(["cargo", "run"])


@functools.cache
def dev_install(release, prof):
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
def lint(release, prof):
    subprocess.run(["cargo", "clippy", "--", "-W", "clippy::pedantic"])


@functools.cache
def fix(release, prof):
    subprocess.run(["cargo", "clippy", "--fix", "--", "-W", "clippy::pedantic"])


@functools.cache
def generate_filters(release, prof):
    with open("src/dsp/filter_coefficients.rs", "w") as f:
        subprocess.run(["python3", "scripts/generate_filters.py"], stdout=f)


if __name__ == "__main__":
    main()
