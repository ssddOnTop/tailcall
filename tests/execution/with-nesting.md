# With nesting

#### server:

```graphql
schema @server @upstream(baseURL: "http://jsonplaceholder.typicode.com") {
  query: Query
}

type Query {
  user: User @http(path: "/users/1")
}

type User {
  id: Int!
  name: String!
  username: String!
  email: String!
  phone: String
  website: String
  posts: [Post] @http(path: "/users/{{value.id}}/posts")
}

type Post {
  id: Int
  title: String
  userId: Int
  body: String
}
```

#### mock:

```yml
- request:
    method: GET
    url: http://jsonplaceholder.typicode.com/users/1
    body: null
  response:
    status: 200
    body:
      email: leanne@mail.com
      id: 1
      name: Leanne Graham
      username: Bret
- request:
    method: GET
    url: http://jsonplaceholder.typicode.com/users/1/posts
    body: null
  response:
    status: 200
    body:
      - title: title1
      - title: title2
      - title: title3
```

#### assert:

```yml
- method: POST
  url: http://localhost:8080/graphql
  body:
    query: query { user { posts { title } } }
```
