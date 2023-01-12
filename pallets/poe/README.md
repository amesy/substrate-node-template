## License
Unlicense

## Function
链上存证功能：存证是一种在线服务，可用于在某一时间点验证计算机文件的存在性，最早是通过比特币网络带有时间戳的交易实现的。    
- 存证的应用场景
    - 数字版权
    - 司法存证
    - 供应链溯源
    - 电子发票
    - …… 

## Implementation
```Bash
pallets/poe
├── Cargo.toml
├── README.md
└── src
    ├── lib.rs
    ├── mock.rs
    └── tests.rs
```

## Notice
相比于v0.9.28, v0.9.35的更新点：
- 1、An RuntimeEvent associated type must be declare on trait `Config`.
- 2、all calls have the `pallet::call_index` attribute or that the `dev-mode` of the pallet is enabled.
  - For more info see: [#12891](https://github.com/paritytech/substrate/pull/12891) and [#11381](https://github.com/paritytech/substrate/pull/11381).
- 3、`mock.rs`： `type Origin = Origin;` -> `type RuntimeOrigin = RuntimeOrigin;`


