# `expie`

This is an experimental GUI framework built with Rust for my own personal usage. Please note it is not quite usable yet.

## I ramble

This library may feel rather unusual in comparison to prior art.

I use fancy words like *terms*, *scopes*, *passes*, etc. to describe some of the concepts, but this is all done in the name of allowing you to extend any part of the framework with your own solutions. Of course, the provided parts are already well integrated *with* each other, and assume you'll be using them *on top of* each other, but I did my best in making it flexible.

Ideally, you shouldn't have to make changes to the library to prototype with your own ideas. Not everything can be adjusted freely, but the framework does give you some room for expression.

In my case, this allows me to add custom features specific to a particular project directly in that project's codebase, without having to edit the framework itself.

I built this with a mindset in opposition to reactive approaches using values-as-signals and so on. It's a wonderful space, but it really is not-my-thing.

No plans for it to ever be "done" as I don't have an end goal, and I will add features as I see fit for my ongoing needs.

If you decide to use this (for **whatever reason**) I also do my best to provide somewhat readable documentation for everything, including private API. I have no stability guarantees to offer at this point. As I said, this exists primarily so that I have something that plays nice with my head.

Have fun...?

## License

Licensed under either of

- Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE) or
  <http://www.apache.org/licenses/LICENSE-2.0>)
- MIT license ([LICENSE-MIT](LICENSE-MIT) or
  <http://opensource.org/licenses/MIT>)

at your option. :3
