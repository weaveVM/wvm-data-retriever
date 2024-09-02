<p align="center">
  <a href="https://wvm.dev">
    <img src="https://raw.githubusercontent.com/weaveVM/.github/main/profile/bg.png">
  </a>
</p>

## About
WeaveVM Data Retriever (`wvm://`) is a WeaveVM data retrieving protocol. It mainly use WeaveVM DA layer and Arweave permanent storage both access to retrieve a WeaveVM transaction data. 

## Build & Run

```bash
git clone https://github.com/weaveVM/wvm-data-retriever.git

cd wvm-data-retriever

cargo shuttle run
```

## Server Methods

### Retrieve calldata associated with an WeaveVM TXID

```bash
curl -X GET https://https://wvm-data-retriever.shuttleapp.rs/calldata/$WVM_TXID
```

Returns

```rs
pub struct HandlerGetCalldata {
    pub calldata: String,
    pub arweave_block_hash: String,
    pub wvm_block_hash: String,
}
```

## License
This project is licensed under the [MIT License](./LICENSE)