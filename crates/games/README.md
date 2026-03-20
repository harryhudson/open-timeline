# `open-timeline-games`

## About

*TODO*

## Testing

```sh
cd crates/games

# Build the WASM (creates /crates/games/pkg)
wasm-pack build --target web

# Serve the test pages
python3 -m http.server
```

Then navigate to `localhost:<port>/test/games/<game>.html` in the browser
