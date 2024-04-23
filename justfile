set windows-shell := ["powershell.exe", "-NoLogo", "-Command"]

show:
    cargo run --bin visualizer

show-release:
    cargo run --bin visualizer --release