<div align="center">

  <h1>Twiggy🌱</h1>

  <strong>A code size profiler for Wasm</strong>

  <p>
    <a href="https://docs.rs/twiggy/"><img src="https://docs.rs/twiggy/badge.svg"/></a>
    <a href="https://crates.io/crates/twiggy"><img src="https://img.shields.io/crates/v/twiggy.svg"/></a>
    <a href="https://crates.io/crates/twiggy"><img src="https://img.shields.io/crates/d/twiggy.svg"/></a>
    <a href="https://travis-ci.org/rustwasm/twiggy"><img src="https://travis-ci.org/rustwasm/twiggy.svg?branch=master"/></a>
  </p>

  <h3>
    <a href="https://rustwasm.github.io/twiggy">Guide</a>
    <span> | </span>
    <a href="https://rustwasm.github.io/twiggy/contributing/index.html">Contributing</a>
    <span> | </span>
    <a href="https://discord.gg/FenCKAEaME">Chat</a>
  </h3>

  <sub>Built with 🦀🕸 by <a href="https://rustwasm.github.io/">The Rust and WebAssembly Working Group</a></sub>
</div>

## About

Twiggy is a code size profiler for Wasm. It analyzes a binary's call graph to
answer questions like:

* Why was this function included in the binary in the first place? Who calls it?

* What is the *retained size* of this function? I.e. how much space would be
  saved if I removed it and all the functions that become dead code after its
  removal.

Use Twiggy to make your binaries slim!

## Install Twiggy

Ensure that you have [the Rust toolchain installed](https://www.rust-lang.org/),
then run:

```
cargo install twiggy --git https://github.com/rustwasm/twiggy
```

## Learn More!

[**Read the Twiggy guide!**](https://rustwasm.github.io/twiggy)

<div align="center">
  <img src="./guide/src/twiggy.png"/>
</div>
