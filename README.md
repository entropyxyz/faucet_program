<!-- Generated with cargo generate entropyxyz/programs -->

# faucet-program

## Running tests

`cargo test`

## Building the program

Get the necessary build tools with:

```shell
cargo install cargo-component --version 0.2.0 &&
cargo install wasm-tools
```

Then build with:

```shell
cargo component build --release --target wasm32-unknown-unknown
```

The `.wasm` binary can be found in `./target/wasm32-unknown-unknown/release`

## Building with docker

If you want to make your program publicly available and open source, and make it possible for others to verify that the source code corresponds to the on-chain binary, you can build it with the included Dockerfile:

```shell
docker build --output=binary-dir .
```

This will compile your program and put the `.wasm` binary file in `./binary-dir`.

## Generate Types

Types are meant top be posted with the program, it is how people know how to interact with your program

They will be autogenerated when running store-programs, or you can run it manually

```shell
cargo run -p generate-types
```

Will generate two files that will hold both the aux_data_schema and config_schema

## Upload program

The basic template is shipped with a cli to upload a program, after compiling the program then generating the types
you upload the program to chain.

Create a .env file with two variables

```env
DEPLOYER_MNEMONIC="<Your Deployer Mnemonic>"
ENTROPY_DEVNET="<Chain Endpoint>"
```

Then run:

```shell
cargo run -p cli -- store-program
```
