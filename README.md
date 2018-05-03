# slicer

A simple, efficient utility for slicing string slices into smaller string 
slices. Useful for parsing anything represented by strings, such as
programming languages or data formats.

## Examples

Basic usage:

```
use slicer::AsSlicer;

let path = "images/cat.jpeg";
let mut slicer = path.as_slicer();

let directory = slicer.slice_until("/");
slicer.skip_over("/");
let filename = slicer.slice_until(".");
slicer.skip_over(".");
let extension = slicer.slice_to_end();

assert_eq!(Some("images"), directory);
assert_eq!(Some("cat"), filename);
assert_eq!(Some("jpeg"), extension);
```

## License

This crate is licensed under the terms of both the MIT License and the Apache License 2.0.