# GitHub Profile Analytics in Rust

A (basic) Rust-based analytics system that tracks views of my GitHub profile using a tracking pixel. Part of my journey to learn Rust.

Built with Actix-web and SQLite, this project demonstrates:

- Async Rust with Actix-web
- SQLite database interactions using SQLx
- JSON API endpoints
- Time-series data handling
- GitHub's Camo proxy logging

Originally deployed with ngrok, this server runs on localhost.

## Note

This project is only a proof of concept. It is not meant to be used in production. There are few security measures in place (only allowing traffic from the Github Camo image proxy), but they are not enough to prevent abuse.

## Live Stats

Live stats are displayed in JSON format through the `/stats` endpoint.

The `/pixel.gif` endpoint is used to track views of the GitHub profile.

## Technical Details

- **Backend**: Rust with Actix-web
- **Database**: SQLite with SQLx
- **Features**:
  - Real-time view tracking
  - Time-based analytics
  - User agent analysis
  - JSON API endpoint for statistics

## Environment Variables

| Variable | Description | Default |
| --- | --- | --- |
| `DATABASE_URL` | Database URL | `sqlite:pulse.db` |
| `HOST` | Host to bind to | `0.0.0.0` |
| `PORT` | Port to bind to | `8080` |

## API Usage

| Endpoint | Description |
| --- | --- |
| `/stats` | Returns the current stats |
| `/pixel.gif` | Tracks views of the GitHub profile (returns a transparent 1x1 pixel) |

Example response for `/stats`:

```json
{
  "total_views": 1234,
  "views_today": 56,
  "views_this_week": 789,
  "views_by_user_agent": [
    {"user_agent": "github-camo", "count": 1000},
    {"user_agent": "Mozilla/5.0...", "count": 234}
  ],
  "recent_views": [
    {
      "timestamp": "2024-01-29T12:34:56Z",
      "camo_id": "d76dea84",
      "user_agent": "github-camo"
    }
  ]
}
```

Example headers in request for `/pixel.gif`:

```json
{
  "user-agent": "github-camo (<hash>)",
  "host": "<host_url>",
  "via": "HTTP/1.1 github-camo (<hash>)",
  "x-forwarded-for": "<github-camo_url>",
  "accept-encoding": "gzip",
  "x-forwarded-proto": "https",
  "accept": "image/webp,image/avif,image/jxl,image/heic,image/heic-sequence,video/*;q=0.8,image/png,image/svg+xml,image/*;q=0.8,*/*;q=0.5",
  "accept-language": "en-US,en;q=0.9",
  "x-forwarded-host": "<host_url>",
}
```

## Database Schema

| Column | Type | Description |
| --- | --- | --- |
| `id` | INTEGER | Primary key |
| `ip_address` | TEXT | IP address of the camo proxy |
| `user_agent` | TEXT | User agent of the request |
| `camo_id` | TEXT | Camo ID of the proxy |
| `timestamp` | DATETIME | Timestamp of the request |


## Potential Next Steps

- Adding rate limiting to the `/pixel.gif` endpoint
- Adding more analytics (geographic distribution of camo proxies, etc.)
- Adding tests (using actix-web's test utilities, creating a mock database, etc.)
- Adding authentication (using a secret key, etc.) for sites outside of GitHub
