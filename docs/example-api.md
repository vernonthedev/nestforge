# Example API Endpoints

Base URL: `http://127.0.0.1:3000`

## Health

- `GET /health`
- Response: `OK`

## Root

- `GET /`
- Response: welcome text

## Users

- `GET /users/`: list users
- `GET /users/{id}`: get one user
- `POST /users/`: create user
- `PUT /users/{id}`: update user

## JSON shapes

Create user body:

```json
{
  "name": "Alice",
  "email": "alice@example.com"
}
```

Update user body:

```json
{
  "name": "Alice Updated",
  "email": "alice2@example.com"
}
```

User response:

```json
{
  "id": 1,
  "name": "Alice",
  "email": "alice@example.com"
}
```

Error response shape:

```json
{
  "statusCode": 400,
  "error": "Bad Request",
  "message": "..."
}
```
