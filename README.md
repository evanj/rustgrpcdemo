# Rust gRPC Demo

This is a demonstration program using Rust with gRPC using Tonic. This interoperates with my [Go gRPC demo](https://github.com/evanj/gogrpcdemo).


# References

* [Tonic Route Guide Tutorial](https://github.com/hyperium/tonic/blob/master/examples/routeguide-tutorial.md)


## Tokio Implementation Notes

`streamclient` has some examples to see how tokio works.

* Sleep future with 0 delay: `poll()` gets called twice: first time it returns Pending, then it returns Ready. It could return Ready right away? I assume the implementation is actually waiting until it returns to the tokio runtime before it is Ready, which makes the zero case more similar to the non-zero case, so maybe is better?

* Stream future with a 0 delay: it gets called three times: the first time it returns Pending, but then something in tonic's implementation has a buffer with data, so it converts it to Ready, which causes it to get polled again. It returns Pending again, because control still has not returned to the tokio runtime. This time it has a zero buffer, so it finally returns to the tokio runtime. It gets polled a third time and returns Ready.
