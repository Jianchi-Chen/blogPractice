# HTTP API Contract

The Axum server is the only source of business data for the browser and Tauri
clients. Both clients use the API below through `frontend/src/api/`. Tauri
commands are reserved for native desktop capabilities such as updates and the
system tray.

## Conventions

- Server origin: `VITE_API_BASE_URL`, without a trailing `/api`.
- Base path: `/api`.
- Authentication: `Authorization: Bearer <token>`.
- IDs: opaque strings. Clients must URL-encode path IDs.
- Timestamps: RFC 3339 strings.
- JSON errors: `{ "code": number, "message": string }`.
- Resource deletes return `204 No Content`; avatar and favorite state operations
  return their authoritative JSON state.
- Temporary aliases for the previously deployed Web bundle live in
  `backend/src/routes/legacy.rs`. They are not a second contract and new code
  must not call them.

## Shared Shapes

```text
AuthResponse  = { token, user_id, username, identity }
CurrentUser   = { id, username, identity, signature, avatar_url }
Article       = { id, title, content, summary, created_at, update_at,
                  status, views, tags }
ArticleSummary = { id, title, summary, created_at, status, views, tags }
Comment       = { comment_id, article_id, user, content, created_at,
                  parent_id, like_count }
CommentWithLike = Comment + { liked_by_me }
```

`identity` is `admin`, `user`, or `visitor`. Article status is `draft`,
`published`, or `archived`. `avatar_url` and optional article/comment fields may
be `null`. `liked_by_me` is `0` or `1` in comment lists and a boolean in a like
response.

## Authentication

| Method | Path | Auth | Request | Success |
| --- | --- | --- | --- | --- |
| `POST` | `/api/auth/register` | optional admin | `{ username, password, identity? }` | `201 AuthResponse` |
| `POST` | `/api/auth/login` | no | `{ username, password }` | `200 AuthResponse` |
| `GET` | `/api/auth/me` | required | none | `200 CurrentUser` |

Public registration creates a `user`. Creating an `admin` or `visitor` requires
an admin bearer token. Usernames are unique.

## Users

| Method | Path | Auth | Request | Success |
| --- | --- | --- | --- | --- |
| `GET` | `/api/users?limit=10` | admin | none | `200 { users: User[] }` |
| `PATCH` | `/api/users/{id}` | admin | `{ username?, password?, identity? }` | `204` |
| `DELETE` | `/api/users/{id}` | admin | none | `204` |
| `PATCH` | `/api/users/me/profile` | required | `{ signature }` | `200 CurrentUser` |
| `PUT` | `/api/users/me/avatar` | required | multipart field `avatar` | `200 CurrentUser` |
| `DELETE` | `/api/users/me/avatar` | required | none | `200 CurrentUser` |
| `GET` | `/api/users/{id}/avatar` | no | none | `200 image/png` or `image/jpeg` |

Signatures are at most 120 characters. Avatars must be valid PNG/JPEG data and
at most 2 MiB. Account `1` cannot be deleted or demoted from admin.

## Articles And Search

| Method | Path | Auth | Request | Success |
| --- | --- | --- | --- | --- |
| `GET` | `/api/articles?query=...` | optional | none | `200 { articles: ArticleSummary[] }` |
| `POST` | `/api/articles` | admin | `Article` fields | `201 Article` |
| `GET` | `/api/articles/{id}` | optional | none | `200 Article` |
| `PUT` | `/api/articles/{id}` | admin | `Article` fields | `200 Article` |
| `PATCH` | `/api/articles/{id}/status` | admin | `{ status }` | `200 Article` |
| `DELETE` | `/api/articles/{id}` | admin | none | `204` |
| `GET` | `/api/search/suggestions?query=...` | no | none | `200 { items: { id, title }[] }` |

Anonymous and non-admin callers only see published articles. Search is
case-insensitive over title, summary, and tags. Suggestions match published
titles only. Creating or replacing an article leaves it as a draft; publishing
is an explicit status update.

## Comments And Likes

| Method | Path | Auth | Request | Success |
| --- | --- | --- | --- | --- |
| `GET` | `/api/articles/{id}/comments` | optional | none | `200 { comments: CommentWithLike[] }` |
| `POST` | `/api/articles/{id}/comments` | user/admin | `{ content, parent_id? }` | `201 Comment` |
| `DELETE` | `/api/comments/{id}` | admin | none | `204` |
| `PUT` | `/api/comments/{id}/like` | user/admin | `{ liked: boolean }` | `200 LikeState` |

Draft and archived article comments follow article visibility: only admins can
read or create them. `LikeState` is `{ comment_id, liked_by_me, like_count }`.
Setting the same like state repeatedly is idempotent and returns the
authoritative count.

## Favorites

| Method | Path | Auth | Request | Success |
| --- | --- | --- | --- | --- |
| `GET` | `/api/favorites` | required | none | `200 { articles: ArticleSummary[] }` |
| `PUT` | `/api/favorites/{article_id}` | required | none | `200 { article_id, favorited: true }` |
| `DELETE` | `/api/favorites/{article_id}` | required | none | `200 { article_id, favorited: false }` |

Only published articles can be added. Add and remove operations are idempotent,
and the list is scoped to the bearer-token user.

## Status Rules

- `400`: malformed or invalid input.
- `401`: missing, expired, invalid, or deleted-user credentials.
- `403`: authenticated caller lacks the required role.
- `404`: resource does not exist or is intentionally hidden from the caller.
- `500`: unexpected server/database failure; internal details are not returned.

The executable contract is covered by `backend/tests/api_contract.rs` and the
frontend HTTP adapter tests in `frontend/src/api/__tests__/`.
