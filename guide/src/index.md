# Twiggy!

[![](https://docs.rs/twiggy/badge.svg)](https://docs.rs/twiggy/)
[![](https://img.shields.io/crates/v/twiggy.svg)](https://crates.io/crates/twiggy)
[![](https://img.shields.io/crates/d/twiggy.svg)](https://crates.io/crates/twiggy)
[![Build Status](https://travis-ci.org/rustwasm/twiggy.svg?branch=master)](https://travis-ci.org/rustwasm/twiggy)

**`twiggy` is a code size profiler.**

It analyzes a binary's call graph to answer questions like:

* Why was this function included in the binary in the first place?

* What is the *retained size* of this function? I.e. how much space would be
  saved if I removed it and all the functions that become dead code after its
  removal.

Use `twiggy` to make your binaries slim!

![Twiggy](./twiggy.png)
