# `expie`

This is an experimental GUI framework built with Rust for my own personal usage.

## I ramble

Please note this library is rather unusual in comparison to prior art.

I use fancy terms to describe some of the concepts, but at the end of the day, this is all done in the name of allowing you to replace any part (layers, terms, dialects) of the framework with your custom solutions. Of course, the provided parts are already well integrated *with* each other, and assume you'll be using them *on top of* each other, but I did my best in making that idea come to life.

Big part of this was the thought that perhaps you shouldn't have to wait for a PR merge, or even have to clone and locally edit the library, to prototype with your own ideas. Not everything can be adjusted freely, but the framework does give you some room for expression.

In my case, analogically, this allows me to add custom features specific to a particular project directly in that project's codebase, without having to edit the framework itself.

This framework exists in opposition to reactive approaches using values-as-signals and so on. It's a wonderful space, but it really is not my thing. You might notice I'm a little opinionated.

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
