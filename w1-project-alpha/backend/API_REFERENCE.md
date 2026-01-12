# Project Alpha - API Reference

Base URL: `http://localhost:3000`

## Ticket Endpoints

### List Tickets
```
GET /api/tickets
```

**Query Parameters:**
- `tag` (optional): UUID - Filter by tag ID
- `search` (optional): string - Search in ticket titles (case-insensitive)
- `completed` (optional): boolean - Filter by completion status

**Response:** `200 OK`
```json
[
  {
    "id": "uuid",
    "title": "string",
    "description": "string | null",
    "completed": boolean,
    "created_at": "timestamp",
    "updated_at": "timestamp",
    "tags": [
      {
        "id": "uuid",
        "name": "string",
        "color": "string | null",
        "created_at": "timestamp"
      }
    ]
  }
]
```

### Get Ticket
```
GET /api/tickets/:id
```

**Response:** `200 OK` or `404 Not Found`
```json
{
  "id": "uuid",
  "title": "string",
  "description": "string | null",
  "completed": boolean,
  "created_at": "timestamp",
  "updated_at": "timestamp",
  "tags": [...]
}
```

### Create Ticket
```
POST /api/tickets
```

**Request Body:**
```json
{
  "title": "string",
  "description": "string (optional)",
  "tag_ids": ["uuid"] // optional
}
```

**Response:** `201 Created`
```json
{
  "id": "uuid",
  "title": "string",
  "description": "string | null",
  "completed": false,
  "created_at": "timestamp",
  "updated_at": "timestamp"
}
```

### Update Ticket
```
PUT /api/tickets/:id
```

**Request Body:** (all fields optional)
```json
{
  "title": "string",
  "description": "string",
  "completed": boolean
}
```

**Response:** `200 OK` or `404 Not Found`
```json
{
  "id": "uuid",
  "title": "string",
  "description": "string | null",
  "completed": boolean,
  "created_at": "timestamp",
  "updated_at": "timestamp"
}
```

### Delete Ticket
```
DELETE /api/tickets/:id
```

**Response:** `204 No Content` or `404 Not Found`

### Toggle Ticket Completion
```
PATCH /api/tickets/:id/toggle
```

**Response:** `200 OK` or `404 Not Found`
```json
{
  "id": "uuid",
  "title": "string",
  "description": "string | null",
  "completed": boolean,
  "created_at": "timestamp",
  "updated_at": "timestamp"
}
```

### Add Tag to Ticket
```
POST /api/tickets/:ticket_id/tags/:tag_id
```

**Response:** `204 No Content` or `404 Not Found`

### Remove Tag from Ticket
```
DELETE /api/tickets/:ticket_id/tags/:tag_id
```

**Response:** `204 No Content` or `404 Not Found`

## Tag Endpoints

### List Tags
```
GET /api/tags
```

**Response:** `200 OK`
```json
[
  {
    "id": "uuid",
    "name": "string",
    "color": "string | null",
    "created_at": "timestamp"
  }
]
```

### Get Tag
```
GET /api/tags/:id
```

**Response:** `200 OK` or `404 Not Found`
```json
{
  "id": "uuid",
  "name": "string",
  "color": "string | null",
  "created_at": "timestamp"
}
```

### Create Tag
```
POST /api/tags
```

**Request Body:**
```json
{
  "name": "string",
  "color": "string (optional)" // hex color like "#ff0000"
}
```

**Response:** `201 Created` or `400 Bad Request` (if name exists)
```json
{
  "id": "uuid",
  "name": "string",
  "color": "string | null",
  "created_at": "timestamp"
}
```

### Update Tag
```
PUT /api/tags/:id
```

**Request Body:** (all fields optional)
```json
{
  "name": "string",
  "color": "string"
}
```

**Response:** `200 OK`, `404 Not Found`, or `400 Bad Request` (if name conflicts)
```json
{
  "id": "uuid",
  "name": "string",
  "color": "string | null",
  "created_at": "timestamp"
}
```

### Delete Tag
```
DELETE /api/tags/:id
```

**Response:** `204 No Content` or `404 Not Found`

## Error Responses

All error responses follow this format:

```json
{
  "error": "error message",
  "status": http_status_code
}
```

**Status Codes:**
- `200 OK` - Successful GET/PUT/PATCH
- `201 Created` - Successful POST
- `204 No Content` - Successful DELETE
- `400 Bad Request` - Validation error
- `404 Not Found` - Resource not found
- `500 Internal Server Error` - Database or server error

## Example Usage

### Create a complete workflow

```bash
# 1. Create tags
curl -X POST http://localhost:3000/api/tags \
  -H "Content-Type: application/json" \
  -d '{"name":"bug","color":"#ff0000"}'

curl -X POST http://localhost:3000/api/tags \
  -H "Content-Type: application/json" \
  -d '{"name":"feature","color":"#00ff00"}'

# 2. Create a ticket with tags (use UUIDs from step 1)
curl -X POST http://localhost:3000/api/tickets \
  -H "Content-Type: application/json" \
  -d '{
    "title":"Fix login issue",
    "description":"Users cannot log in",
    "tag_ids":["uuid-of-bug-tag"]
  }'

# 3. Get all tickets
curl http://localhost:3000/api/tickets

# 4. Search tickets
curl "http://localhost:3000/api/tickets?search=login"

# 5. Filter by tag
curl "http://localhost:3000/api/tickets?tag=uuid-of-bug-tag"

# 6. Toggle completion
curl -X PATCH http://localhost:3000/api/tickets/{ticket-uuid}/toggle

# 7. Add another tag
curl -X POST http://localhost:3000/api/tickets/{ticket-uuid}/tags/{feature-tag-uuid}

# 8. Update ticket
curl -X PUT http://localhost:3000/api/tickets/{ticket-uuid} \
  -H "Content-Type: application/json" \
  -d '{"title":"Update login flow","completed":true}'

# 9. Delete ticket
curl -X DELETE http://localhost:3000/api/tickets/{ticket-uuid}
```

## Notes

- All timestamps are in ISO 8601 format with timezone
- UUIDs are version 4 UUIDs
- Tag names must be unique
- Deleting a ticket automatically removes all its tag associations
- Deleting a tag automatically removes it from all tickets
- Search is case-insensitive and uses PostgreSQL's ILIKE
- Multiple query parameters can be combined for complex filtering
