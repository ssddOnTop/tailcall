# test-enum

###### check identity

#### server:

```graphql
schema @server @upstream(baseURL: "http://jsonplaceholder.typicode.com") {
  query: Query
}

enum Foo {
  BAR
  BAZ
}

type Query {
  foo: Foo @http(path: "/foo")
}
```
