# Trellis
______________________________________________________________________

Trellis is a batteries-included command runner, designed to make it simple to write code for numerical calculations. It aims to remove boilerplate from user-code, allowing you to focus on core application logic. It aims to separate code for business logic from support code. Features include:
* Automatic emission of tracing messages.
* Support for caller termination through `ctrl-c` and `tokio`.
* Timing

In future, we hope to implement:
* Writing of progress and results to disk
* Plotting of incremental progress and results

The design of `trellis` is influenced by the command runner within the excellent `argmin` crate, but aims to provide a lightweight, application agnostic framework with a minimal dependency tree.
