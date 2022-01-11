# Exchange Order-Book Merge Stream

## Usage
```text
order-book-merger 0.1.0
Dan Aharon <dan@aharon.dev>
Order Book Merger

USAGE:
    order-book-merger [OPTIONS] <SYMBOL>

FLAGS:
        --help       Prints help information
    -V, --version    Prints version information

OPTIONS:
    -h, --host <HOSTNAME>      IP address to listen on [default: 127.0.0.1]
    -l, --log-level <LEVEL>    Log level (TRACE, DEBUG, ERROR, WARN, INFO). [default: info]
    -p, --port <PORT>          Port number to listen on [default: 8080]

ARGS:
    <SYMBOL>    The trading symbol, eg. 'ethbtc'
```

## Test
```shell
cargo test
```

## Run Server
```shell
cargo run -- ethbtc
```

## Client
Use [BloomRPC](https://github.com/bloomrpc/bloomrpc) gRPC GUI client.   
