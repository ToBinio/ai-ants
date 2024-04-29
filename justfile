set windows-shell := ["powershell.exe", "-NoLogo", "-Command"]

@show *args='':
    cargo run --bin visualizer $@

@train *args='':
    cargo run --bin trainer $@