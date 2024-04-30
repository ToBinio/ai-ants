set windows-shell := ["powershell.exe", "-NoLogo", "-Command"]

@show *path:
    cargo run --bin visualizer --release -- {{path}}

@train *args='':
    cargo run --bin trainer {{args}}