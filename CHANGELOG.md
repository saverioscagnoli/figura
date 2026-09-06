# Changes from v1.3.2 are documented below.

v1.3.2 - August 14 2025

- Added `Str` variant to the `Value` enum by [dheijl](https://github.com/dheijl)

v1.3.3 - December 14 2025

- Improve performance when using a lot of numeric tokens by [dheijl](https://github.com/dheijl)
- Implement clippy suggestions by [dheijl](https://github.com/dheijl)

v2.0.0 - January 15 2026

- Library rewriting with better separation of concerns
- Removed arbitrary alignment support
- Removed `SwitchDirective`
- Template creation function changed from "parse" to "compile"

V2.0.3 - January 15 2026
- Replace `ryu` with `zmij`

V3.0.0 - July 08 2026
- Now the directives inside the templates must be Send + Sync, which could break previous implementations.
- Updated depedencies to latest versions.

V3.1.0 - September 06 2026
- Removed `itoa`, replaced with std functions by [dheijl](https://github.com/dheijl)
- Replaced README.
