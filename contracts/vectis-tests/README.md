# Vectis Contract Tests

This is an integration tests workspace.
Allows us to clearly layout all single and multiple chain integration tests between contracts and ibc calls

Some naming details:

- `DAO chain`: where the VectisDAO is, i.e. Juno Network
- `remote chain`: where remote factory, remote proxy and remote tunnel contracts are deployed,
  communicates with VectisDAO via IBC

- `single-chain/`: Integration tests for single chain interactions using the `cw-plus/multi-test` testing library
  - chain proxy instantiation, migration, update multisig and Govec
- `multi-chain/`: Integration tests for multiple chain interactions
  - **NOTE** `cw-plus/multi-test` does not support IBC as of v0.15.1 and therefore whilst we use it,
    IBC interactions are done with direct mocks calling `#entry-points`
