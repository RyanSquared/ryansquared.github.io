---
layout: post
title: 'Error Handling in Rust: eyre versus thiserror'
---

Oftentimes, when I'm discussing the choice of using Rust with someone, they
will have points that I think are entirely valid about why they should not
choose Rust as their language of choice. One of those points is that error
handling in Rust is a monster and a half. I believe this is due to the way that
languages in other ecosystems have previously designed errors as well as the
fundamental difference between what I will call "application error handling"
and "library error handling".

When someone uses an application and they encounter an issue with what they are
attempting to do, it is common to give them a quick, easy-to-understand message
about what went wrong, and sometimes a context that a user, without inner
knowledge of how the application works, can take advantage of to resolve the
situation. This is "application error handling", and I personally use the Rust
crate `color-eyre` when managing these errors. `color-eyre` builds upon the
foundation of the `eyre` crate, offering a `Report` with `Sections`. Let's take
a basic example:

```sh
cargo new --bin facility
cd facility
cargo add color-eyre
```

```rust
// src/main.rs
use color_eyre::{eyre::WrapErr, Result, Section};

fn main() -> Result<()> {
    color_eyre::install()?;

    let config_file = std::fs::read_to_string("config.json")
        .wrap_err("config file could not be read: config.json")
        .suggestion("try copying the example config: `cp config.example.json config.json`")?;

    Ok(())
}
```

The `?` postfix operator will either return a `Result::Err` from the function
or will evaluate to an unwrapped `Result::Ok` value. It can automatically
convert an error into the expected error type, but because the `install`
method, `wrap_error` method, and `suggestion` methods all return a `Report`,
they are not implicitly converted. However, if we wanted to, we could
automatically convert all errors into `color_eyre` Results by using the `?`
postfix operator instead of using `wrap_err`.

This example gives the user of the application insight on what can be done to
resolve the situation. We're given the context of the error (a file couldn't be
read) with a solution to the most common problem, the file not existing, which
can be fixed by copying the example file. From the perspective of a user, this
is the ideal error. It provides enough information that a solution can be 
reached without providing intricate knowledge of what's happening under the
hood. The user doesn't have to learn anything about the inner workings of the
program they're using to resolve the error.

However, if you're a library developer, these kinds of errors can be
infuriating because they don't provide the correct amount of context. All they
convey is a surface-level overview of the information, without providing the
information about what can actually cause the error. Instead, the type of error
handling that is common for libraries, "library error handling", typically
revolves around a type that encodes information about what could have caused
the error, and what situation arose that could help with resolving the error.
As an example, let's look at the `Error` type of `serde_json`, a popular
library for serializing and deserializing objects. It provides some methods
that give more context about what failed and when it failed. The
`serde_json::Error` type does not expose the fields directly, but it does offer
some key context about what could happen: `line()` and `column()` methods and a
`classify()` method that describes what type of error could have occurred.

Let's use the crate `thiserror` to provide a wrapper around the `serde_json`
error type, so we can provide context as to whether an error is an IO error or
a serde error. We'll make use of the `#[from]` attribute for `thiserror`, which
can create a `From` implementation for one error into our new error, as well as
the `#[error()]` attribute, which allows us to quickly implement a `Display`
implementation for our error.

```sh
cargo add thiserror
cargo add serde_json
```

```rust
// main.rs
use std::fs::File;
use std::io::BufReader;
use std::path::Path;

use color_eyre::{Result, Section};
use serde_json::Value;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum Error {
    #[error("config file could not be read: {0}")]
    Io(#[from] std::io::Error),

    #[error("config could not be deserialized: {0}")]
    Json(#[from] serde_json::Error),
}

fn load_config_from_file(config_file: impl AsRef<Path>) -> Result<Value, Error> {
    let file = File::open(config_file)?;
    let reader = BufReader::new(file);
    let config = serde_json::from_reader(reader)?;
    Ok(config)
}

fn main() -> Result<()> {
    color_eyre::install()?;

    let config = load_config_from_file("config.json")
        .suggestion("try copying the example config: `cp config.example.json config.json`")?;

    println!("config: {config:?}");

    Ok(())
}
```

This example makes use of both `color_eyre` and `thiserror`, with `thiserror`
providing context as to which step could have failed, and `color_eyre`
providing a user-level suggestion as to how the problem can be resolved. Most
times, you can get away with using just `color_eyre` or `thiserror` in a crate,
but this was a good opportunity to demonstrate the strengths of both solutions.

In summary, I think that the two styles of error handling serve their own
individual purposes, and while confusing to newcomers, I believe that the two
distinct styles can provide a significant advantage over languages that provide
only one style of error handling. Some languages, such as Python and
JavaScript, have a trend of providing a single string's worth of context. This
does not provide enough information to a programmer who may be debugging an
application, and may provide too much technical information to a user for whom
the info isn't necessary. However, providing too much information to the user
of an application can end up obscuring a potential fix. The common styles of
Rust error handling provide a significant amount of information that is useful
to programmers while also providing the key details about an error to the
users.
