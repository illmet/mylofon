# Mylofon

A lightweight, monolithic social platform clone written in Go.

## Features

- **Auth**: Claim-based identity (no passwords, just claim ID + secret key)
- **Posts**: 280-character microblogging
- **Profiles**: Customizable avatar and header
- **Timeline**: Chronological feed
- **Tech Stack**: Go (Chi), SQLite (WAL mode), Redis (Sessions/Rate limits), HTMX (Frontend)

## Prerequisites

To run this application as a standalone monolith, you need:

1. **Go 1.25+**: To build the application
2. **GCC**: Required for SQLite (CGO)
3. **Redis**: Must be installed and running locally on port 6379 (default)

## Running Locally

1. **Start Redis**
   Ensure your local Redis server is running:
   ```bash
   redis-server
   ```

2. **Run the Application**
   ```bash
   # Install dependencies
   go mod download
   
   # Run the server
   go run cmd/server/main.go
   ```

   The server will start at `http://localhost:8080`.

## Building for Production

To build a single binary:

```bash
# Build the binary
go build -o mylofon cmd/server/main.go

# Run it
./mylofon
```

*Note: You must still have a Redis instance accessible to the binary via `REDIS_URL` environment variable if not on localhost.*
