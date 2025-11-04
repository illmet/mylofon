# Mylofon

A simple Twitter clone with basic posting and reaction functionality.

## Features

- **Post Creation**: Share thoughts in 280 characters or less
- **Three Reaction Types**:
  - 🎬 **Kino**: Quality/cinema-worthy content
  - 📚 **Info**: Informative/educational content
  - 🗑️ **Slop**: Low-quality content
- **Infinite Scroll**: Seamlessly load more posts as you scroll
- **Real-time Updates**: Reaction counts update immediately

## Tech Stack

**Frontend:**
- React + TypeScript
- Vite (fast build tool)
- Tailwind CSS (utility-first styling)

**Backend:**
- Node.js + Express + TypeScript
- SQLite (lightweight, embedded database)
- RESTful API

## Getting Started

### Prerequisites

- Node.js 18+ installed
- npm or yarn package manager

### Installation

1. Install server dependencies:
```bash
cd server
npm install
```

2. Install client dependencies:
```bash
cd ../client
npm install
```

### Running the Application

1. Start the backend server (from the `server` directory):
```bash
npm run dev
```
Server will run on http://localhost:3001

2. Start the frontend (from the `client` directory):
```bash
npm run dev
```
Client will run on http://localhost:3000

3. Open your browser and navigate to http://localhost:3000

### Seeding the Database (Optional)

To populate the database with 240 philosophical quotes from 20 famous thinkers:

```bash
cd server
npm run seed
```

This will add sample posts from philosophers and authors including Socrates, Plato, Aristotle, Nietzsche, Kafka, Camus, Virginia Woolf, Rumi, and many more!

### Usage

1. Enter a username in the top-right corner (stored in localStorage)
2. Type a post in the composer and click "Post"
3. React to posts by clicking the reaction buttons
4. Scroll down to load more posts automatically

## Authentication (PoC)

Currently using a simple username field for proof-of-concept. No passwords or authentication required.

### Future Auth Options:

- **Clerk**: Modern, user-friendly auth (recommended for rapid setup)
- **Auth0**: Enterprise-grade authentication
- **NextAuth.js**: Flexible, open-source solution
- **Custom JWT**: Full control, requires more setup

## Database Schema

**Posts Table:**
- id (primary key)
- username
- content (max 280 chars)
- created_at

**Reactions Table:**
- id (primary key)
- post_id (foreign key)
- username
- reaction_type (kino/info/slop)
- created_at
- UNIQUE constraint on (post_id, username)

## API Endpoints

- `GET /api/posts?limit=20&offset=0&username=user` - Get posts with pagination
- `POST /api/posts` - Create a new post
- `POST /api/reactions` - Add/update/remove reaction

## Scaling Considerations

The architecture is designed for minimal bloat while supporting future scaling:

1. **Database**: Easy migration from SQLite to PostgreSQL
2. **Frontend**: Component-based architecture for easy feature additions
3. **API**: RESTful design, can be extended or replaced with GraphQL/tRPC
4. **Deployment**: Can be containerized with Docker

## License

MIT
