## Sparse merkle tree rocksdb store implementation
This is a rust implementation of a [sparse merkle tree](https://github.com/nervosnetwork/sparse-merkle-tree) store using rocksdb as the backend.

### Usage
Please refer to the unit tests for usage examples.

### Examples

#### Start a rocksdb store backed sparse merkle tree

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

#### Start a rocksdb store backed with multiple sparse merkle trees

Please be aware that the tree name is used as the prefix of the key to store the leaf and branch, so the tree name should be unique. Otherwise, the data of the tree with the same prefix name may be overwritten. Suggest to add a fixed suffix to the tree name to make it unique, for example. add a `.` to the tree name, like "tree." and "tree1."

Or you may use the `ColumnFamilyStore` to replace the `DefaultStore` in the `rpc_server_multi_tree.rs` example, which will use two different column families to store the smt branch and leaf data.

```
cargo run --example rpc_server_multi_tree -- /tmp/smt-store-dir 127.0.0.1:10000
```

call rpc server to update the tree
```
echo '{
    "id": 2,
    "jsonrpc": "2.0",
    "method": "update_all",
    "params": [
        "tree1.",
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
        "tree1.",
        ["2b2b2b2b2b2b2b2b2b2b2b2b2b2b2b2b2b2b2b2b2b2b2b2b2b2b2b2b2b2b2b2b", "1111111111111111111111111111111111111111111111111111111111111111"]
    ]
}' \
| curl -H 'content-type: application/json' -d @- \
http://localhost:10000
```

call rpc server to clear a tree:
```
echo '{
    "id": 2,
    "jsonrpc": "2.0",
    "method": "clear",
    "params": [
        "tree1."
    ]
}' \
| curl -H 'content-type: application/json' -d @- \
http://localhost:10000
```
