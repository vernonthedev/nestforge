# Example API Endpoints

Base URL: `http://127.0.0.1:3000`

The example app uses global prefix `api`, so routes start with `/api`.

## Root

- `GET /api/`
- Response: welcome string

## Health

- `GET /api/health`
- Response: `OK`

- `GET /api/health/db`
- Response: `DB_READY`

## Users (v1)

- `GET /api/v1/users/`
- `GET /api/v1/users/count`
- `GET /api/v1/users/{id}`
- `GET /api/v1/users/{id}/exists`
- `POST /api/v1/users/`
- `PUT /api/v1/users/{id}`
- `PUT /api/v1/users/{id}/replace`
- `DELETE /api/v1/users/{id}`

### Create User Body

```json
{
  "name": "Alice",
  "email": "alice@example.com"
}
```

### Update User Body

```json
{
  "name": "Alice Updated",
  "email": "alice2@example.com"
}
```

## Settings (v1)

- `GET /api/v1/settings/runtime`
- `GET /api/v1/settings/`
- `GET /api/v1/settings/{id}`
- `POST /api/v1/settings/`
- `PUT /api/v1/settings/{id}`
- `DELETE /api/v1/settings/{id}`

## Versioning Demo

- `GET /api/v1/versioning/hello`
- `GET /api/v2/versioning/hello`

## Error Shape

NestForge errors return a consistent JSON shape:

```json
{
  "statusCode": 400,
  "error": "Bad Request",
  "message": "..."
}
```
