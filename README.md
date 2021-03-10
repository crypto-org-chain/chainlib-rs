chainlib is a library to help with creating HD wallets and signing [Crypto.org Chain](https://github.com/crypto-org-chain/chain-main) transfer transactions offline.

# prepare
Before a test, we need to send some coin amount to a HD wallet recovered from the mnemonic words.

1. transfer some coin to a mnemonic words-recovered account; to get the HD address, you can use the following code:
```rust
let words = "dune car envelope chuckle elbow slight proud fury remove candy uphold puzzle call select sibling sport gadget please want vault glance verb damage gown";
let mnemonic = Mnemonic::from_str(words, password)?;
let key_service = PrivateKeyService::new_from_mnemonic(mnemonic)?;
let address = key_service.address()?;
let address_str = address.to_bech32("cro");
println!("{}", address_str);
```
or you can recover mnemonic to a local storage:
`chain-maind keys add hd-wallet --keyring-backend test --recover`
and use `chain-maind keys list --keyring-backend test` to see the address.

2. Transfer some coin amount to the recovered HD address using `chain-maind`:
```
chain-maind tx bank send \
    ${from_address} \
    ${hd_address} \
    100cro \
    --keyring-backend test \
    --chain-id test \
    --sign-mode amino-json
```

3. There are some coins in the HD wallet now, you can test to sign offline and send the signed transaction to Chain API URL. To see the full detail, go to the examples directory.

# build examples
`cargo build --example amino`

or 

`cargo build --example protobuf --features=grpc`

# unit test
`cargo test --lib --all-features`