set windows-shell := ["powershell.exe", "-NoLogo", "-Command"]

@show *args='':
    cargo run --bin visualizer {{args}}

@train *args='':
    cargo run --bin trainer {{args}}