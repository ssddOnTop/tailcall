# Grpc datasource

#### file:news.proto

```protobuf
syntax = "proto3";

import "google/protobuf/empty.proto";

package news;

message News {
    int32 id = 1;
    string title = 2;
    string body = 3;
    string postImage = 4;
}

service NewsService {
    rpc GetAllNews (google.protobuf.Empty) returns (NewsList) {}
    rpc GetNews (NewsId) returns (News) {}
    rpc GetMultipleNews (MultipleNewsId) returns (NewsList) {}
    rpc DeleteNews (NewsId) returns (google.protobuf.Empty) {}
    rpc EditNews (News) returns (News) {}
    rpc AddNews (News) returns (News) {}
}

message NewsId {
    int32 id = 1;
}

message MultipleNewsId {
    repeated NewsId ids = 1;
}

message NewsList {
    repeated News news = 1;
}
```

#### server:

```graphql
schema
  @server(port: 8000, graphiql: true)
  @upstream(httpCache: true, batch: {delay: 10}, baseURL: "http://not-a-valid-grpc-url.com")
  @link(id: "news", src: "news.proto", type: Protobuf) {
  query: Query
}

type Query {
  news: NewsData! @grpc(method: "news.NewsService.GetAllNews", baseURL: "http://localhost:50051")
  newsById(news: NewsInput!): News!
    @grpc(method: "news.NewsService.GetNews", body: "{{args.news}}", baseURL: "http://localhost:50051")
}
input NewsInput {
  id: Int
  title: String
  body: String
  postImage: String
}
type NewsData {
  news: [News]!
}

type News {
  id: Int
  title: String
  body: String
  postImage: String
}
```

#### mock:

```yml
- request:
    method: POST
    url: http://localhost:50051/news.NewsService/GetAllNews
    body: null
  response:
    status: 200
    body: \0\0\0\0t\n#\x08\x01\x12\x06Note 1\x1a\tContent 1\"\x0cPost image 1\n#\x08\x02\x12\x06Note 2\x1a\tContent 2\"\x0cPost image 2
```

#### assert:

```yml
- method: POST
  url: http://localhost:8080/graphql
  body:
    query: query { news {news{ id }} }
```
