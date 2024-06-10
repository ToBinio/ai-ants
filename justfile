set windows-shell := ["powershell.exe", "-NoLogo", "-Command"]

show path:
    cargo run --bin simulation-visualizer --release -- -p {{path}}

show-random:
    cargo run --bin simulation-visualizer --release

train *args='':
    cargo run --bin trainer --release -- {{args}}

clear:
    rm -rf ./training

show-network path:
    cargo run --bin neural-network-visualizer --release -- -p {{path}}
