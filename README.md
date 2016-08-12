# SpongeJNI

This repo is a proof-of-concept. It allows for [Sponge](https://www.spongepowered.org/) plugins to
have a majority of their code implemented via JNI, in this case through
[Rust](http://rust-lang.org/).

This is done by generating Rust `struct`s through examining the SpongeAPI class files, allowing for
native Rust calls, which access the SpongeAPI through JNI.

A plugin is necessary to act as a sort of shim: calling native code when necessary, generating
bytecode, etc.

Note that running this in its current state will crash the JVM, since Sponge appears to not have
full support for achievements.

## Building

Clone the repo and enter it.

`mvn clean package`

Copy the shaded jar to the Sponge mods directory.

Enter the `rust` directory.

`cargo build`

Copy the resulting dylib to the Sponge root directory.

Start Sponge.

## Usage

If you join the server, the JVM will crash. To fix this, comment out the **body** of both listeners
in `listeners.rs`.

The `rusty` command will greet you by your name.
