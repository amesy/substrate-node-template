# 作业

![task](https://p.qlogo.cn/qqmail_head/fpxD8nQbaFSOgAU3ZYYC5JustPs9dA3135iaEb7Tbzwol1uVEqry1RA0YRzFm1C45/0)

# 作业完成情况
创建学生信息，并存储到链上，将学生的附加信息如个人档案PDF等在链下存储,经更新后重新通过发交易的方式推送到链上。

![](https://p.qlogo.cn/qqmail_head/W2z2eH7ppFco2u6A77RR8yoicyicyicpCHOLh6vYSIVkhXibY7ma37zCT4oGXTc1ZLZR/0)
![](https://p.qlogo.cn/qqmail_head/W2z2eH7ppFco2u6A77RR8yoicyicyicpCHOLh6vYSIVkhVvibBZmicUu52mc9dQJgsPSh/0)

链上随机数（如前面Kitties示例中）与链下随机数的区别？

- [链上随机数](https://docs.rs/pallet-randomness-collective-flip/3.0.0/pallet_randomness_collective_flip/)是基于前81个区块哈希生成的低影响随机值。在较低频情况下使用时比较有用，反之则不太靠谱，主要用在诸如测试之类的低安全性情况下；
- [链下随机数](https://docs.rs/sp-io/6.0.0/sp_io/offchain/fn.random_seed.html)在链下执行，由基于链下主机环境产生的真正随机的，非确定性的种子，在OCW环境下更具随机性。

（可选）在 Offchain Worker 中，解决向链上发起不签名请求时剩下的那个错误。
- 参考：https://github.com/paritytech/substrate/blob/master/frame/examples/offchain-worker/src/lib.rs




