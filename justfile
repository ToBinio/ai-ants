set windows-shell := ["powershell.exe", "-NoLogo", "-Command"]

show path:
    cargo run --bin visualizer --release -- -p {{path}}

show-random:
    cargo run --bin visualizer --release

train *args='':
    cargo run --bin trainer --release -- {{args}}

clear:
    rm -rf ./training
