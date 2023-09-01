# GuoQL

锅 bot，但是 GraphQL。

## 用法

```text
Usage: guoql [OPTIONS] [DB]

Arguments:
  [DB]  数据文件路径。 [default: ./guoql.db]

Options:
      --host <HOST>  监听地址。 [default: 127.0.0.1]
      --port <PORT>  监听端口。 [default: 8080]
  -d, --debug        打开调试日志。
  -h, --help         Print help
```

启动后，将在子路由 `/graphql` 上同时开启 GraphQL 服务（`POST`）和 GraphQL Playground（`GET`）。

## GraphQL Schema

```graphql
schema {
  query: Query
  mutation: Mutations
}

type Query {
  # 有锅吗？
  pots: [Pot!]!

  # 查询一个锅。
  pot(id: Int, index: Int): Pot!

  # 查询统计。
  stats(top: Int): [EaterStats!]!
}

type Pot {
  # 锅的 ID。
  id: Int!

  # 锅哪啊？
  position: String!

  # 啥时候锅啊？
  time: String!

  # 啥口味啊？
  taste: String!

  # 谁锅啊？
  eaters: [Eater!]!

  # 备注。
  note: String

  # 一共几面？
  mian: Int!

  # 一共几饭？
  fan: Int!
}

type Eater {
  # 你谁啊？
  name: String!

  # 几面？
  mian: Int!

  # 几饭？
  fan: Int!
}

type Mutations {
  # 约锅。
  newPot(
    position: String!
    time: String!
    taste: String!
    name: String!
    mian: Int!
    fan: Int!
    note: String
  ): Pot!

  # 吃锅。
  eat(id: Int, index: Int, name: String!, mian: Int!, fan: Int!): Pot!

  # 吃完了。
  finish(id: Int, index: Int): Pot!

  # 改锅。
  edit(
    id: Int
    index: Int
    position: String
    time: String
    taste: String
    note: String
  ): Pot!

  # 下车。
  leave(id: Int, index: Int, name: String!): Pot!

  # 改需求。
  editDemand(id: Int, index: Int, name: String!, mian: Int, fan: Int): Pot!

  # 清空锅。
  clear: [Pot!]!
}

type EaterStats {
  # 你谁啊？
  name: String!

  # 总面数。
  mian: Int!

  # 总饭数。
  fan: Int!

  # 吃锅次数。
  eatCount: Int!

  # 约锅次数。
  potCount: Int!

  # 平均面数。
  avgMian: Float!

  # 平均饭数。
  avgFan: Float!
}
```