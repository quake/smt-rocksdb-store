## Sparse merkle tree rocksdb store implementation
This is a rust implementation of a [sparse merkle tree](https://github.com/nervosnetwork/sparse-merkle-tree) store using rocksdb as the backend.

### Usage
Please refer to the unit tests for usage examples.

### Example
Start a rocksdb store backed sparse merkle tree

```
cargo run --example rpc_server -- /tmp/smt-store-dir 127.0.0.1:10000
```

call rpc server to update the tree
```
echo '{
    "id": 2,
    "jsonrpc": "2.0",
    "method": "update_all",
    "params": [
        [
            ["2a2a2a2a2a2a2a2a2a2a2a2a2a2a2a2a2a2a2a2a2a2a2a2a2a2a2a2a2a2a2a2a", "2a2a2a2a2a2a2a2a2a2a2a2a2a2a2a2a2a2a2a2a2a2a2a2a2a2a2a2a2a2a2a2a"],
            ["2b2b2b2b2b2b2b2b2b2b2b2b2b2b2b2b2b2b2b2b2b2b2b2b2b2b2b2b2b2b2b2b", "2b2b2b2b2b2b2b2b2b2b2b2b2b2b2b2b2b2b2b2b2b2b2b2b2b2b2b2b2b2b2b2b"],
            ["2d2d2d2d2d2d2d2d2d2d2d2d2d2d2d2d2d2d2d2d2d2d2d2d2d2d2d2d2d2d2d2d", "ffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff"],
            ["1111111111111111111111111111111111111111111111111111111111111111", "eeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeee"],
            ["3333333333333333333333333333333333333333333333333333333333333333", "dddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddd"]
        ]
    ]
}' \
| curl -H 'content-type: application/json' -d @- \
http://localhost:10000
```

call rpc server to get the proof:
```
echo '{
    "id": 2,
    "jsonrpc": "2.0",
    "method": "merkle_proof",
    "params": [
        ["2b2b2b2b2b2b2b2b2b2b2b2b2b2b2b2b2b2b2b2b2b2b2b2b2b2b2b2b2b2b2b2b", "1111111111111111111111111111111111111111111111111111111111111111"]
    ]
}' \
| curl -H 'content-type: application/json' -d @- \
http://localhost:10000
```
