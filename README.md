[![Version](https://img.shields.io/crates/v/michie)][crates.io]
[![License](https://img.shields.io/crates/l/michie)][license]
![Downloads](https://img.shields.io/crates/d/michie)
![Recent downloads](https://img.shields.io/crates/dr/michie)
[![CI status](https://img.shields.io/github/actions/workflow/status/mobusoperandi/michie/ci.yml?branch=master)][ci]

michie (sounds like Mickey) — an attribute macro that adds [memoization] to a function.

<!-- TOC -->
# Table of contents

1. [Features](#features)
1. [Non-features](#non-features)
1. [key_expr](#key_expr)
1. [store_type](#store_type)
1. [store_init](#store_init)
1. [Type requirements](#type-requirements)
    1. [General bounds](#general-bounds)
    1. [Store bounds](#store-bounds)
1. [Generic functions](#generic-functions)
1. [Functions that take no input](#functions-that-take-no-input)
1. [How it works](#how-it-works)
1. [Why must key_expr be provided](#why-must-key_expr-be-provided)
1. [Support and feedback](#support-and-feedback)
<!-- TOC -->

# Features

- Supports
    - Plain functions
    - Generic functions
    - Functions in `impl` blocks
    - Functions in trait implementation blocks
    - Functions that are default trait implementations
- Thread safe
- Expansion depends on only `std`
- Hygienic
- Supports recursive functions
- Bring your own store

# Non-features

- Caching features: this crate does not provide a caching mechanism other than some trivial implementations.
  It allows you to bring your own.
- "Blazingly fast": this crate aims to provide a simple and easy-to-use means of memoizing a function.
  If you *actually really* require micro-optimized memoization then you'd most likely have to implement it yourself.

# key_expr

In each invocation a key is obtained.
It is used to query the function's cache store for a possible hit.
An expression that evaluates into a key must be provided via the `key_expr` argument.
The expression may use bindings from the function's parameters.
In the following example the `key_expr` is simply the name of the only parameter.

```rust
use michie::memoized;
use std::collections::HashMap;
#[memoized(key_expr = input, store_type = HashMap<usize, usize>)]
fn f(input: usize) -> usize {
    // expensive calculation
    # unimplemented!()
}
```

# store_type

A concrete store type must be either provided via the `store_type` argument or inferred from the `store_init` (next section).

The provided type must implement [`MemoizationStore`].
Implementations of [`MemoizationStore`] for [`BTreeMap`] and [`HashMap`] are provided.
In the following example, [`BTreeMap`] is provided as the store:

```rust
use michie::memoized;
use std::collections::BTreeMap;
#[memoized(key_expr = input, store_type = BTreeMap<usize, usize>)]
fn f(input: usize) -> usize {
    // expensive calculation
    # unimplemented!()
}
```

# store_init

By default, the store is initialized via [`Default::default()`].
Different initialization may be provided via an expression to `store_init`:

```rust
use michie::{memoized, MemoizationStore};
use std::collections::HashMap;
#[memoized(key_expr = input, store_init = HashMap::<usize, usize>::with_capacity(500))]
fn f(input: usize) -> usize {
    // expensive calculation
    # unimplemented!()
}
```

# Type requirements

Bounds apply to the key type and the function's return type.
Some are from the general instrumentation and others are via the store type's implementation of [`MemoizationStore`].

## General bounds

The following apply to the key type and to the function's return type:

- [`Sized`]: for one, the instrumentation stores the key in a `let` binding.
- [`'static`]: key and return values are owned by a store which is owned by a static.
- [`Send`] and [`Sync`]: for parallel access.

## Store bounds

Another source of bounds on the key type and the return type is the implementation of [`MemoizationStore`] for the store type.

# Generic functions

Be mindful of the [type requirements](#type-requirements) when using on a generic function:

```rust
use michie::memoized;
use std::hash::Hash;
use std::collections::HashMap;
#[memoized(key_expr = input.clone(), store_type = HashMap<A, B>)]
fn f<A, B>(input: A) -> B
where
    A: 'static + Send + Sync // bounds from instrumentation
        + Eq + Hash // store-specific bounds
        + Clone, // used in this function's `key_expr`
    B: 'static + Send + Sync + Clone // bounds from instrumentation
        + From<A>, // used in this function's body
{
    input.into()
}
```

# Functions that take no input

Functions that take no input are good candidates for [compile-time evaluation],
which is usually preferred over runtime caching (such as this crate provides).
Nonetheless, some functions cannot be evaluated at compile time.
A reasonable `key_expr` for a function that takes no input is `()`:

```rust
use michie::memoized;
use std::collections::HashMap;
#[memoized(key_expr = (), store_type = HashMap<(), f64>)]
fn f() -> f64 {
    // expensive calculation
    # unimplemented!()
}
```

# How it works

The original function expands into something similar to this:

```rust ignore
fn f(input: Input) -> Output {
    static STORE = Mutex::new(#store_init);
    let key = #key_expr;
    let store_mutex_guard = STORE.lock().unwrap();
    let attempt = store_mutex_guard.get(&key).cloned();
    drop(store_mutex_guard);
    if let Some(hit) = attempt {
        return hit;
    } else {
        let miss = #original_fn_body;
        STORE.lock().unwrap().insert(key, miss.clone());
        return miss;
    };
}
```

# Why must key_expr be provided

The only conceivable default `key_expr` is the entire input.
For example, for a function signature:
```text
fn f(a: usize, _b: usize) -> usize
```
the default `key_expr` would be `(a, _b)`.
Two potential problems: some parameters might not satisfy [bounds on the key type](#type-requirements).
Also, the resulting key might be a supervalue of _the input of the_ __actual__ _calculation_.
To explain the latter problem, here is an example:

```rust
use michie::memoized;
use std::collections::HashMap;
// pretend that `key_expr` is omitted and that this is the default
#[memoized(key_expr = (a, _b), store_type = HashMap<(usize, usize), usize>)]
fn f(a: usize, _b: usize) -> usize {
    // the actual calculation uses a subvalue of the input — only `a`
    # a
}
f(0, 0); // expected miss because it's the first invocation
f(0, 1); // avoidable miss!
```

If an accurate `key_expr = a` had been provided, the second execution would have been a hit.
To summarize, `key_expr` is mandatory in order to encourage proper consideration of it.

# Support and feedback

In [the GitHub Discussions].

[`'static`]: https://doc.rust-lang.org/rust-by-example/scope/lifetime/static_lifetime.html#trait-bound
[compile-time evaluation]: https://doc.rust-lang.org/std/keyword.const.html#compile-time-evaluable-functions
[memoization]: https://en.wikipedia.org/wiki/Memoization
[crates.io]: https://crates.io/crates/michie
[ci]: https://github.com/mobusoperandi/michie/actions/workflows/ci.yml
[license]: https://tldrlegal.com/license/mit-license
[the GitHub Discussions]: https://github.com/mobusoperandi/michie/discussions
