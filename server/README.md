# Server

Server is responsible to organize the performance measurement tests for each client connected to it. Measured
performance data is then mapped to a reliability score and stored in a hashmap.

## Build Guide

To start the server, run the following command:
```bash
RUST_LOG=info cargo run
```
