# Reliability Measurement server
Reliability measurement server measures CPU and network performance of each client connected to it via websocket and map it to
client reliability score. This score can be used by other services to customize their interaction with different clients depending 
upon their score.

## Build Guide

### Server

```
RUST_LOG=info cargo run -p server
```

### Client

An implementation of client is not provided, but it can be built easily by using `shared` crate.

## How client reliability score is calculated?

Reliability server measures CPU and network performance of each client connected to it via websocket and maps it to client reliability score. This score can be used by other services to customize their interaction with different clients depending upon their score. 

Performance measurement tests are carried out multiple times to cancel out volatility. 

### Single thread CPU performance measurement

For CPU power measurement, we are using timelock puzzle to measure
CPU's raw processing capability. 

It has the following advantages:
1) Puzzle is `intrinsically sequential`, so there is no known way to calculate it in parallel
2) If secret primes are known, it is easy to verify correctness of the solution.

Reference: http://bitsavers.trailing-edge.com/pdf/mit/lcs/tr/MIT-LCS-TR-684.pdf

### Network I/O performance measurement

For network I/O measurement, we are measuring round-trip time for the configurable size of the data. Data is generated cryptographically secure RNG so that it cannot be cached.

### Limitation

Since client will be running a Webassembly code in browser using structures defined in shared crate, depending upon vendor and settings performance can vary significantly.
