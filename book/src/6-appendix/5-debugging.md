
# Debugging
Toql generates a lot of code. Mostly from the Toql derive, but also from various macros, such as `query!`.

Toql does not have any seriuos software tests and the test matrix is huge. So it may happen 
- that you hit a bug.
- the generated code doesn't compile.

Or you just want to develop a new feature!

To debug Toql generated code,

1. If you have a lot of mods  move the affected `mod` at the end of the mod list. (So generated code will appear in the terminal last).
2. Run cargo with the logger enabled and a single job:
```rust
 RUST_LOG=DEBUG cargo check --jobs=1
```
3. Copy all the code from the Toql derive and paste it into the source file.
4. Remove the log header by regex replacing `\[2.*` with empty string. There should be 9 occurences.
5. Copy your struct.
6. Comment out your orgiginal struct.
7. On the copied struct remove all references to Toql.
8. Format your document and debug!

## Suport
If you have issues with Toql you can post them on [GitHub](https://github.com/roy-ganz/toql/issues).

