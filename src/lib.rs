//! Provides IP address related types and data structures for the Rust programming language with a
//! clean and complete API and many nice features. The basic types are opaque, immutable,
//! comparable, space efficient, and defined as simple structs that don't require extra memory
//! allocation.
//!
//! First, a little history: I write a lot of go code in my job and I do a lot of virtual
//! networking. I work with IP Addresses regularly. I was unhappy with the implementation of IP
//! addresses [in the go standard library][gonet], and a few other go libraries, for a number of
//! reasons, so [I wrote my own][goaddrs].
//!
//! For now, programming in Rust is more a part of my hobby-life than my work-life but I wanted to
//! replicate the functionality of my go library in Rust for two reasons: because I think it could
//! enrich Rust and I want a good learning experience.
//!
//! [goaddrs]: https://pkg.go.dev/gopkg.in/addrs.v1
//! [gonet]: https://pkg.go.dev/net#IP

pub mod ipv4;
