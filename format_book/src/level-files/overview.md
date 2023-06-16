# Format overview
The level format consists of a hierarchical tree of nodes (also called chunks). At least on PC, the files are little endian. Each node follows the basic format:
 - **u8[???]** - Potential padding. Sometimes data is padded with zeroes between nodes, and that padding must be skipped.
 - **u8[4]** - Node's identifier. It can be any arbitrary set of 4 bytes. Often it's ASCII characters, but it can also be 32-bit values like hashes.
 - **u32** - Payload size in bytes.
 - **u8[payload_size]** - Actual payload, which may be either:
   - Raw data of whatever is meant to be stored in the node.
   - Child nodes.
   - Child nodes with a `u32` prefix specifying the amount of children.

There's no way to know what kind of data is stored in a node, unless you either hardcode the behavior or try probing the node's contents to see whether it follows a hierarchical pattern.

## `lvl_` nodes and hashing
Data files that are meant to be selectively loaded by Lua scripts tend to be made out of nodes that follow the following structure:
 - `lvl_`
   - `weird hash`
     - usual data, textures, models, etc.

This weird hash is nothing more than a hashed name of the resource. For example, for `all_fly_snowspeeder`, that hash is going to be `0x266561d8`.

The hash algorihm used is 32-bit FNV-1a ([Wikipedia](https://en.wikipedia.org/wiki/Fowler%E2%80%93Noll%E2%80%93Vo_hash_function), [official (?) website](http://www.isthe.com/chongo/tech/comp/fnv/)), which can be defined as:

```rs
const FNV_PRIME: u32 = 16777619;
const OFFSET_BASIS: u32 = 2166136261;

fn fnv1a_hash(buffer: &[u8]) -> u32 {
    let mut result = OFFSET_BASIS;
    for &byte in buffer {
        // NOTE: BF2 additionally ORs every byte with 0x20, to make the encoding
        //       case insensitive, even if it does technically break it a little
        result ^= (byte | 0x20) as u32;
        result = result.wrapping_mul(FNV_PRIME);
    }
    result
}
```

## Config nodes
TODO: describe config nodes
