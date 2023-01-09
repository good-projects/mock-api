## request data of `params`, `body` and `headers`

All these data act as conditions, it means it will be always matched if leave
all these data empty.

### Using raw data

```json
{
  "params": {
    "name": "foo"
  }
}
```

### Using relational operators

operators: `is`, `is!`, `contains`, `contains!`

### Example

```json
{
  "project": "my-project",
  "endpoints": [
    {
      "path": "hello",
      "when": [
        {
          "method": "GET",
          "request": {
            "params": [
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
