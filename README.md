### Note

Avoid using third-party libraries as much as possible.

## queries, headers, body of request

These data are used to match the request data.

## Example

operators: `is`, `is!`, `contains`, `contains!`

```json
{
  "description": "my-project",
  "endpoints": [
    {
      "path": "hello",
      "when": [
        {
          "method": "GET",
          "request": {
            "queries": [
              {
                "operator": "is!",
                "name": "name",
                "value": "foo"
              }
            ],
            "headers": {
              "token": "go"
            }
          },
          "response": {
            "status": 200,
            "body": {},
            "headers": {}
          },
          "delay": 400
        },
        {
          "method": "POST",
          "request": {
            "body": {
              "name": "foo"
            },
            "headers": {
              "content-type": "xxx"
            }
          },
          "response": {
            "status": 200,
            "body": {},
            "headers": {}
          }
        }
      ]
    }
  ]
}
```
